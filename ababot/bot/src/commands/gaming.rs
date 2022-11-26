use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        interaction::{
            application_command::ApplicationCommandInteraction, InteractionResponseType,
        },
        ChannelId,
    },
    prelude::Context,
};
use tracing::instrument;

use crate::utils::get_channel_id;

#[instrument(skip(ctx, command))]
pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) {
    let mut duration = None;
    let mut unit = None;
    let mut target_channel = None;

    for option in &command.data.options {
        if option.name == "duration" {
            duration = option.value.as_ref().and_then(|v| v.as_f64())
        }
        if option.name == "unit" {
            unit = option
                .value
                .as_ref()
                .and_then(|v| v.as_str().map(|s| s.to_string()))
        }

        if option.name == "channel" {
            target_channel = option
                .value
                .as_ref()
                .and_then(|v| v.as_str())
                .map(|name| get_channel_id(name, ctx.http.as_ref()))
        }
    }

    match (duration, unit, target_channel) {
        (Some(d), Some(u), Some(t)) => {
            let text_response = format!("Moving you back in {}{}", d, u);
            let d = if u == "m" { d * 60.0 } else { d * 60.0 * 60.0 };
            match move_channel_users(d, t.await, command, ctx).await {
                Ok(_) => {
                    if let Err(why) = command
                        .create_interaction_response(&ctx.http, |response| {
                            response
                                .kind(InteractionResponseType::ChannelMessageWithSource)
                                .interaction_response_data(|message| message.content(text_response))
                        })
                        .await
                    {
                        tracing::warn!("Failed to run command: {}", why);
                    }
                }
                Err(_) => {
                    if let Err(why) = command
                        .create_interaction_response(&ctx.http, |response| {
                            response
                                .kind(InteractionResponseType::ChannelMessageWithSource)
                                .interaction_response_data(|message| {
                                    message
                                        .content("Something went wrong while executing command")
                                        .ephemeral(true)
                                })
                        })
                        .await
                    {
                        tracing::warn!("Failed to run command: {}", why);
                    }
                }
            }
        }
        _ => {
            tracing::debug!("Ignoring malformed request, missing duration or unit");
            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message
                                .content("Both duration and unit are required to use this command")
                                .ephemeral(true)
                        })
                })
                .await
            {
                tracing::warn!("Failed to run command: {}", why);
            }
        }
    }
}

#[instrument(skip(command, ctx))]
async fn move_channel_users(
    d: f64,
    target_channel: Result<ChannelId, &str>,
    command: &ApplicationCommandInteraction,
    ctx: &Context,
) -> Result<(), ()> {
    let mut original_channel = None;
    for channel in command
        .guild_id
        .ok_or(())?
        .channels(&ctx.http)
        .await
        .map_err(|_e| ())?
    {
        if !channel.1.is_text_based() {
            for user in channel.1.members(&ctx.cache).await.unwrap() {
                if command.member.as_ref().ok_or(())?.user.id == user.user.id {
                    original_channel = Some(channel.clone());
                }
            }
        }
    }
    let original_channel = original_channel.ok_or(())?.0;
    let target_channel = target_channel.map_err(|_e| ())?;
    let cache = ctx.cache.clone();
    let http = ctx.http.clone();

    tokio::spawn(async move {
        for user in original_channel
            .to_channel((&cache, http.as_ref()))
            .await
            .unwrap()
            .guild()
            .unwrap()
            .members(&cache)
            .await
            .unwrap()
        {
            let r = user.move_to_voice_channel(&http, target_channel).await;
            if r.is_err() {
                tracing::warn!("Failed to move user {:?}", user.nick)
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs_f64(d)).await;

        for user in target_channel
            .to_channel((&cache, http.as_ref()))
            .await
            .unwrap()
            .guild()
            .unwrap()
            .members(&cache)
            .await
            .unwrap()
        {
            let r = user.move_to_voice_channel(&http, original_channel).await;
            if r.is_err() {
                tracing::warn!("Failed to move user {:?}", user.nick)
            }
        }
    });

    Ok(())
}

#[instrument(skip(command))]
pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    tracing::debug!("Registering command game");
    command
        .name("game")
        .description("Play a game for a limited duration")
        .create_option(|option| {
            option
                .name("duration")
                .description("How long should the gaming session be")
                .kind(serenity::model::prelude::command::CommandOptionType::Number)
                .min_number_value(0.0)
                .required(true)
        })
        .create_option(|option| {
            option
                .name("unit")
                .description("Time unit for duration")
                .kind(serenity::model::prelude::command::CommandOptionType::String)
                .add_string_choice("minutes", "m")
                .add_string_choice("hours", "h")
                .required(true)
        })
        .create_option(|option| {
            option
                .name("channel")
                .description("The channel to game in")
                .kind(serenity::model::prelude::command::CommandOptionType::Channel)
                .required(true)
        })
}
