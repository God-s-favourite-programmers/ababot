use std::env;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool};

use serenity::async_trait;
use serenity::model::application::interaction::Interaction;
use serenity::model::prelude::{GuildId, Ready, ChannelId};
use serenity::prelude::{Context, EventHandler};

pub mod commands;
pub mod types;
pub mod utils;

pub struct Handler {
    pub loop_running: AtomicBool,
}

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
        println!("Bot ready");
        tracing::info!("Connecting as {}", ready.user.name);

        // Check should not be neccessary as ready is only called once
        // Utenfor makro
        let ctx = Arc::new(ctx);

        // Begyn makro element
        // En thread for hver bakgrunnsaktivitet
        // Må ha unikt navn per element
        let ctx_copy = Arc::clone(&ctx);
        tokio::spawn(async move {
            ChannelId(772092284153757719).send_message(&ctx_copy.http, |m| {
                    m.embed(|e| {
                        e.title("Asyncly doing shit")
                        .field("Async", "Async is coool", false)
                    })
                }).await
            });
        // Slutt makro element

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
            Ok(_) => {tracing::debug!("Command registration succeeded")}
            Err(e) => {eprintln!("{:?}", e); tracing::error!("Command registration failed: {:?}", e)},
        }
        tracing::info!("Setup complete");
    }
}
