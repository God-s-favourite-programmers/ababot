use std::env;

use serenity::model::prelude::{Ready, GuildId};
use serenity::prelude::{Context, EventHandler};
use serenity::{async_trait};
use serenity::model::application::interaction::{Interaction, InteractionResponseType};

pub mod commands;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            tracing::debug!("Reveeiced command interaction {:#?}", command);

            let input = command.data.name.as_str();
            let content = dir_macros::invoke!("bot/src/commands" "run(&command.data.options)");

            if let Err(why) = command.create_interaction_response(&ctx.http, |response| {
                response.kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.content(content))
            }).await {
                tracing::warn!("Failed to run command {}: {}", input, why);
            }

        }

    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        tracing::info!("Connected as {}", ready.user.name);

        let guild_id = GuildId(
            env::var("GUILD_ID")
                .expect("Expected GUILD_ID in environment")
                .parse()
                .expect("GUILD_ID must be an integer"),
        );

        let commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
            commands.create_application_command(|command| commands::ping::register(command))
        }).await;

        tracing::info!("Guild commands created: {:#?}", commands);
    }
}