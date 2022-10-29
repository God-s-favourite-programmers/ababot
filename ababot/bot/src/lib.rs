use std::env;

use serenity::async_trait;
use serenity::model::application::interaction::Interaction;
use serenity::model::prelude::{GuildId, Ready};
use serenity::prelude::{Context, EventHandler};

pub mod commands;
pub mod database;
pub mod types;
pub mod utils;

pub struct Handler;

async fn nop() {}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let input = command.data.name.as_str();

            tracing::debug!("Executing command {input}");
            dir_macros::run_commands_async!("bot/src/commands" "commands" "run(&ctx,&command)");
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        tracing::info!("Connecting as {}", ready.user.name);

        let guild_id = GuildId(
            env::var("GUILD_ID")
                .expect("Expected GUILD_ID in environment")
                .parse()
                .expect("GUILD_ID must be an integer"),
        );

        tracing::debug!("Got Guild Id: {}", &guild_id);

        let commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
            dir_macros::register_commands!("bot/src/commands" "commands" "register(command)")
        }).await;

        match commands {
            Ok(_) => {
                tracing::debug!("Command registration succeeded")
            }
            Err(e) => {
                eprintln!("{:?}", e);
                tracing::error!("Command registration failed: {:?}", e)
            }
        }
        tracing::info!("Setup complete");
    }
}
