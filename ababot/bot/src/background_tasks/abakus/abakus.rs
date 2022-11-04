use std::sync::Arc;

use chrono::Datelike;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use reqwest::Client;
use serenity::{model::prelude::ChannelId, prelude::Context};
use tokio::time::sleep;

use crate::background_tasks::abakus::types::{ApiEvent, Event};
use crate::utils::{get_channel_id, schedule, Time, WEEK_AS_SECONDS};

const EVENT_URL: &str = "https://abakus.no/events/";
pub async fn run(ctx: Arc<Context>) {
    //TODO: spawn another thread to watch for reactions to messages
    let today = chrono::offset::Local::now().date();
    let tomorrow = today.succ();

    schedule(
        Time::EveryDeltaStartAt(
            std::time::Duration::from_secs(WEEK_AS_SECONDS),
            tomorrow.and_hms(8, 0, 0),
        ),
        || async { fetch_and_send(ctx.clone()).await },
    )
    .await
}

pub async fn fetch_and_send(ctx: Arc<Context>) {
    let channel_id = match get_channel_id("abakus-events", &ctx.http).await {
        Ok(id) => id,
        Err(e) => {
            tracing::error!("Failed to get channel id: {}", e);
            return;
        }
    };

    let fetched_data = match fetch().await {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("Could not fetch events. Reason: {}", e);
            return;
        }
    };

    let all_events = parse_events(fetched_data);

    let filtered_events = filter_existing_messages(ctx.clone(), all_events).await;

    for event in filtered_events {
        let channel_message = ChannelId(channel_id.0)
            .send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.title(&event.title)
                        .description(&event.description)
                        .field(
                            "Time",
                            &event.event_time.to_rfc2822().split("+").next().unwrap(),
                            false,
                        )
                        .field("Where", &event.event_location, false)
                        .url(format!("{}{}", EVENT_URL, event.id))
                        .image(&event.thumbnail)
                        .footer(|f| f.text(&event.id))
                })
            })
            .await;
        if let Err(e) = channel_message {
            tracing::warn!("Could not send message. Reason: {}", e);
        }
        sleep(std::time::Duration::from_secs(2)).await;
    }
}

async fn fetch() -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new();
    let today = chrono::Local::today();
    let url = format!(
        "https://lego.abakus.no/api/v1/events/?date_after={}",
        today.format("%Y-%m-%d")
    );
    tracing::info!("Fetching events from {}", url);
    let res = client.get(url).send().await?.text().await?;
    Ok(res)
}

fn parse_events(events: String) -> Vec<Event> {
    let v: serde_json::Value = serde_json::from_str(&events).unwrap();
    let results = v["results"].as_array().unwrap();
    let events: Vec<ApiEvent> = results
        .into_par_iter()
        .map(|e| {
            let id = e["id"].as_i64().unwrap() as i32;
            let title = e["title"].as_str().map(|s| s.to_string());
            let description = e["description"].as_str().map(|s| s.to_string());
            let event_time = e["startTime"].as_str().map(|s| s.to_string());
            let event_location = e["location"].as_str().map(|s| s.to_string());
            let thumbnail = e["cover"].as_str().map(|s| s.to_string());
            ApiEvent {
                id,
                title,
                description,
                event_time,
                event_location,
                thumbnail,
            }
        })
        .collect();

    events.into_iter().map(|e| e.into()).collect()
}

async fn filter_existing_messages(ctx: Arc<Context>, events: Vec<Event>) -> Vec<Event> {
    let channel_id = match get_channel_id("abakus-events", &ctx.http).await {
        Ok(id) => id,
        Err(e) => {
            tracing::error!("Failed to get channel id: {}", e);
            return Vec::new();
        }
    };
    let embeds = ChannelId(channel_id.0)
        .messages(&ctx.http, |m| m.limit(100))
        .await
        .unwrap()
        .into_iter()
        .map(|m| m.embeds)
        .flatten()
        .collect::<Vec<_>>();
    let footers: Vec<String> = embeds
        .into_iter()
        .map(|e| e.footer)
        .flatten()
        .map(|f| f.text)
        .collect();
    events
        .into_par_iter()
        .filter(|e| !footers.contains(&e.id.to_string()))
        .collect()
}
