use std::{env, sync::{atomic::AtomicBool, Arc}};

use bot::{utils::{self, gpgpu::gpu::gpu_handler}, Handler};
use serenity::{prelude::GatewayIntents, Client};

#[tokio::main]
async fn main() {
    let (subscriber, _guard) = utils::get_logger();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global default subscriber");
    tracing::trace!("Log setup complete");

    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    // Build our client.
    let mut client = Client::builder(
        token,
        GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT,
    )
    .event_handler(Handler {
        loop_running: AtomicBool::new(false),
    })
    .await
    .expect("Error creating client");

    {
        let mut data = client.data.write().await;
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<utils::gpgpu::channels::GPU>(100);

        let sender_arc = Arc::new(sender);
        data.insert::<utils::gpgpu::channels::GPU>(sender_arc);
        tokio::spawn(async move {
            if (gpu_handler(&mut receiver)).await.is_err() {
                tracing::error!("GPU handler failed");
            }
        });
    }

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        tracing::error!("Error running client: {why}");
    }
}
