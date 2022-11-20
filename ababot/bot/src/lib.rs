use std::env;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use serenity::async_trait;
use serenity::model::application::interaction::Interaction;
use serenity::model::prelude::{GuildId, Ready};
use serenity::prelude::{Context, EventHandler};
use tokio::sync::mpsc;

use crate::utils::gpgpu::channels::GPU;
use crate::utils::gpgpu::gpu::gpu_handler;

pub mod background_tasks;
pub mod commands;
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
        tracing::info!("Connecting as {}", ready.user.name);

        // Check should not be neccessary as ready is only called once
        // Utenfor makro
        let (sender, mut receiver) = mpsc::channel::<GPU>(100);
        tokio::spawn(async move {
            if (gpu_handler(&mut receiver).await).is_err() {
                tracing::error!("GPU handler failed");
            }
        });

        let sender = Arc::new(sender);
        let ctx = Arc::new(ctx);
        // Every background task has to handle its own setup, executing, and contiguos execution
        dir_macros::long_running!("bot/src/background_tasks" "background_tasks" "run(ctx_cpy, sender_cpy)");
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
                tracing::error!("Command registration failed: {:?}", e)
            }
        }
        tracing::info!("Setup complete");
        println!("Bot ready");
    }
}
