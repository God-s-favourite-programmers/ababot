use std::sync::Arc;

use serenity::{model::prelude::ChannelId, prelude::Context};
use tokio::time::sleep;

pub async fn run(ctx: Arc<Context>) {
    // Wait til 13:00 local time

    //TODO: spawn another thread to watch for reactions to messages

    loop {
        let now = chrono::Local::now();
        let mut target = chrono::Local::today().and_hms(8, 55, 1);
        if now > target {
            target = target + chrono::Duration::days(1);
        }
        let duration = (target - now).to_std().unwrap();
        // This is early wakeup to begin fetching
        sleep(duration).await;

        // TODO: fetch

        //TODO: premake message

        //TODO: Sleep til 9:00

        // TODO fix message to be from abakus
        let channel_message = ChannelId(772092284153757719)
            .send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.title("Akakus")
                        .field("Will send at 13:02", "TADA", false)
                })
            })
            .await;
        if let Err(e) = channel_message {
            // TODO: log
            println!("Error: {:?}", e);
    }
    }
}
