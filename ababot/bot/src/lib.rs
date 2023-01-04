use std::env;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use serenity::async_trait;
use serenity::model::application::interaction::Interaction;
use serenity::model::prelude::{GuildId, Ready};
use serenity::prelude::{Context, EventHandler};
use tokio::fs::create_dir;
use tracing::instrument;

use crate::commands::kok::save_big;
use crate::utils::background_threads::ThreadStorage;

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
        } else if let Interaction::ModalSubmit(submit) = interaction {
            let modal = submit.data.custom_id.as_str();
            match modal {
                "kok" => {
                    tracing::debug!("Executing modal {modal}");
                    save_big(&ctx, &submit).await;
                }
                &_ => {
                    tracing::debug!("Modal {modal} not handled");
                }
            }
        } else {
            tracing::debug!("Interaction not handled");
        }
    }

    #[instrument(skip(self, ctx, ready))]
    async fn ready(&self, ctx: Context, ready: Ready) {
        tracing::info!("Connecting as {}", ready.user.name);

        // Kok folder setup
        create_dir("kok")
            .await
            .unwrap_or_else(|_| panic!("Could not create kok folder"));

        // Check should not be neccessary as ready is only called once
        let ctx = Arc::new(ctx);
        let mut data = ctx.data.write().await;
        let running = data
            .get::<ThreadStorage>()
            .expect("ThreadCounter not found in data")
            .running;
        if !running {
            // Every background task has to handle its own setup, executing, and contiguous execution
            dir_macros::long_running!("bot/src/background_tasks" "background_tasks" "run(ctx_cpy)");
            data.insert::<ThreadStorage>(Arc::new(ThreadStorage { running: true }));
        } else {
            tracing::info!("Background tasks already running");
        }
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
