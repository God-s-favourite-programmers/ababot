use chrono_tz::Europe::Oslo;
use serenity::{
    prelude::Context,
};

use std::sync::Arc;

use crate::utils::{get_channel_id, schedule, DAY_AS_SECONDS, Time};
// example link: https://api.e24.no/content/v1/comics/2022-11-21 
const URL: &str = "https://api.e24.no/content/v1/comics/";

pub async fn run(ctx: Arc<Context>) {

    let now = chrono::Utc::now().with_timezone(&Oslo);
    let today = now.date_naive();
    let today_start = today.and_hms_opt(20, 32, 0);
    match today_start {
        Some(start) => {
            let start = start
                .and_local_timezone(Oslo)
                .earliest()
                .unwrap_or_else(|| {
                    tracing::error!("Could not convert start time to timezone");
                    panic!("Could not convert start time to timezone");
                });
            let date = if start < now {
                now.date_naive().succ_opt().unwrap_or_else(|| {
                    tracing::error!("Could not get tomorrow's date");
                    panic!("Could not get tomorrow's date");
                })
            } else {
                now.date_naive()
            };

            schedule(
                Time::EveryDeltaStartAt(
                    std::time::Duration::from_secs(DAY_AS_SECONDS),
                    date.and_hms_opt(20, 32, 0)
                        .unwrap_or_else(|| {
                            tracing::error!("Could not set time for start date");
                            panic!("Could not set time for start date");
                        })
                        .and_local_timezone(Oslo)
                        .earliest()
                        .unwrap_or_else(|| {
                            tracing::error!("Could not convert start time to timezone");
                            panic!("Could not convert start time to timezone");
                        }),
                ),
                || async { get_lunch(ctx.clone()).await },
            )
            .await
        }
        None => {
            tracing::error!("Could not generate start time");
            panic!("Could not generate start time")
        }
    }
}

async fn get_lunch(ctx: Arc<Context>){
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
            e.title("Dagens LUNCH")
            .image(url)
        })
    }).await;

    if let Err(e) = message {
        tracing::warn!("Could not send lunch: {}", e);
        return;
    }
}