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
            target_channel = option.value.as_ref().and_then(|v| v.as_str())
        }
    }

    match (duration, unit, target_channel) {
        (Some(d), Some(u), Some(t)) => {
            let text_response = format!("Moving you back in {}{}", d, u);
            let d = if u == "m" { d * 60.0 } else { d * 60.0 * 60.0 };
            match move_channel_users(d, t, command, ctx).await {
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
                Err(u) => {
                    let text = match u {
                        true => {"Something went wrong, are you in a VC I have access to?"},
                        false => {"Something went wrong while executing the command"}
                    };

                    if let Err(why) = command
                        .create_interaction_response(&ctx.http, |response| {
                            response
                                .kind(InteractionResponseType::ChannelMessageWithSource)
                                .interaction_response_data(|message| {
                                    message
                                        .content(text)
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

type UserError = bool;

#[instrument(skip(command, ctx))]
async fn move_channel_users(
    d: f64,
    target_channel: &str,
    command: &ApplicationCommandInteraction,
    ctx: &Context,
) -> Result<(), UserError> {
    let cache = ctx.cache.clone();
    let http = ctx.http.clone();
    let guild = command
        .guild_id
        .ok_or_else(|| {
            tracing::warn!("Could not retreive guild id");
            false
        })?
        .to_guild_cached(&cache)
        .ok_or_else(|| {
            tracing::warn!("Could not retreive guild struct");
            false
        })?;

    let original_channel = guild
        .voice_states
        .get(&command.user.id)
        .ok_or_else(|| {
            tracing::debug!("User is not in a VC");
            true
        })?
        .channel_id
        .ok_or(false)?;
    let target_channel = ChannelId(target_channel.parse().map_err(|e| {
        tracing::error!("Failed to parse channel ID: {:?}", e);
        false
    })?);

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
            if let Err(e) = r {
                tracing::warn!(
                    "Failed to move user {:?} to {:?}: {}",
                    user.nick,
                    target_channel,
                    e
                );
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
