use chrono_tz::Europe::Oslo;
use serenity::prelude::Context;

use std::{sync::Arc, time::Duration};

use crate::utils::{time::{Time, schedule, Interval, DAY_AS_SECONDS}, get_channel_id};

// example link: https://api.e24.no/content/v1/comics/2022-11-21
const URL: &str = "https://api.e24.no/content/v1/comics/";
const START_TIME: (u8, u8, u8) = (7, 0, 0);

pub async fn run(ctx: Arc<Context>) {
    let t = Time::new_unchecked(START_TIME.0, START_TIME.1, START_TIME.2).nearest_unchecked();
    schedule(
        Interval::EveryDeltaStartAt(Duration::from_secs(DAY_AS_SECONDS), t),
        || async { get_lunch(ctx.clone()).await },
    )
    .await;
}

async fn get_lunch(ctx: Arc<Context>) {
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

    let message = channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| e.title("Dagens LUNCH").image(url))
        })
        .await;

    if let Err(e) = message {
        tracing::warn!("Could not send lunch: {}", e);
    }
}
