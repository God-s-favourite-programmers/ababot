use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        command::CommandOptionType,
        interaction::{
            application_command::ApplicationCommandInteraction, InteractionResponseType,
        },
    },
    prelude::Context,
};
use tracing::instrument;

use crate::database::{load::load_all_data, save::save};


#[instrument(skip(ctx, command))]
pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) {
    let mut ans = "No option selected".to_string();
    for option in &command.data.options {
        if option.name == "save" {
            let save_string = option
                .value
                .as_ref()
                .map(|v| v.as_str())
                .flatten()
                .unwrap_or("default");
            ans = match save(String::from(save_string)).await {
                Ok(_) => format!("Saved to {}", save_string),
                Err(_) => format!("Failed to save to {}", save_string),
            };
        }
        if option.name == "load" {
            ans = load_all_data().await.unwrap();
        }
    }
    if let Err(why) = command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.content(ans))
        })
        .await
    {
        tracing::warn!("Failed to run command: {}", why);
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    tracing::debug!("Registering command knock");
    command
        .name("db")
        .description("Database testing")
        .create_option(|option| {
            option
                .name("save")
                .description("Query to run")
                .kind(CommandOptionType::String)
                .required(false)
        })
        .create_option(|option| {
            option
                .name("load")
                .description("Query to run")
                .kind(CommandOptionType::String)
                .required(false)
        })
}
