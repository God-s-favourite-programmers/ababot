use chrono_tz::Europe::Oslo;
use serenity::{
    model::{channel::EmbedImage, prelude::ChannelId},
    prelude::Context,
};

use std::sync::Arc;

use crate::utils::get_channel_id;
// example link: https://api.e24.no/content/v1/comics/2022-11-21 
const URL: &str = "https://api.e24.no/content/v1/comics/";

pub async fn run(ctx: Arc<Context>) {
    let now = chrono::Utc::now().with_timezone(&Oslo);
    let today = now.date_naive();
    let url = format!("{}{}", URL, today.format("%Y-%m-%d"));

    let channel_id = match get_channel_id("lunch", &ctx.http).await {
        Ok(id) => id,
        Err(e) => {
            tracing::error!("Failed to get channel id: {}", e);
            return;
        }
    };

    let message = channel_id.send_message(&ctx.http, |m|{
        m.embed(|e|{
            e.image(url)
        })
    }).await;
}
