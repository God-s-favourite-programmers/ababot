use std::sync::Arc;

use chrono::{DateTime, Utc};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serenity::{model::prelude::ChannelId, prelude::Context};
use tokio::time::sleep;

use crate::utils::{schedule, Time};

const _EVENT_URL: &str = "https://abakus.no/events/";
#[derive(Serialize, Deserialize, Debug)]
struct ApiEvent {
    title: Option<String>,
    description: Option<String>,
    event_time: Option<String>,
    event_location: Option<String>,
    thumbnail: Option<String>,
}
#[derive(Debug)]
struct Event {
    title: String,
    description: String,
    event_time: DateTime<Utc>,
    event_location: String,
    thumbnail: String,
}

impl From<ApiEvent> for Event {
    fn from(api_event: ApiEvent) -> Self {
        Event {
            title: api_event.title.unwrap_or_else(|| "No title".to_string()),
            description: api_event
                .description
                .unwrap_or_else(|| "No description".to_string()),
            event_time: api_event
                .event_time
                .unwrap_or_else(|| "".to_string())
                .parse()
                .unwrap_or_else(|_| Utc::now()),
            event_location: api_event
                .event_location
                .unwrap_or_else(|| "N/A".to_string()),
            thumbnail: api_event.thumbnail.unwrap_or_else(|| "N/A".to_string()),
        }
    }
}

pub async fn fetch_and_send(ctx: Arc<Context>) {
    let fetched_data = match fetch().await {
        Ok(v) => v,
        Err(e) => {
            println!("Error: {:?}", e);
            return;
        }
    };

    let events = parse_events(fetched_data);

    for event in events {
        let channel_message = ChannelId(772092284153757719)
            .send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.title(&event.title)
                        .description(&event.description)
                        .field("Time", &event.event_time.to_rfc2822(), false)
                        .field("Where", &event.event_location, false)
                        .image(&event.thumbnail)
                })
            })
            .await;
        if let Err(e) = channel_message {
            println!("Error: {:?}", e);
        }
        sleep(std::time::Duration::from_secs(2)).await;
    }
}

pub async fn run(ctx: Arc<Context>) {
    //TODO: spawn another thread to watch for reactions to messages
    schedule(Time::EveryTime(chrono::offset::Local::now().date().and_hms(8, 0, 0)), || async {
        fetch_and_send(ctx.clone()).await
    }).await
}

async fn fetch() -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new();
    let today = chrono::Local::today();
    let url = format!(
        "https://lego.abakus.no/api/v1/events/?date_after={}",
        today.format("%Y-%m-%d")
    );
    println!("Fetching {}", url);
    let res = client.get(url).send().await?.text().await?;
    Ok(res)
}

fn parse_events(events: String) -> Vec<Event> {
    let v: serde_json::Value = serde_json::from_str(&events).unwrap();
    let results = v["results"].as_array().unwrap();
    let events: Vec<ApiEvent> = results
        .into_par_iter()
        .map(|e| {
            let title = e["title"].as_str().map(|s| s.to_string());
            let description = e["description"].as_str().map(|s| s.to_string());
            let event_time = e["startTime"].as_str().map(|s| s.to_string());
            let event_location = e["location"].as_str().map(|s| s.to_string());
            let thumbnail = e["cover"].as_str().map(|s| s.to_string());
            ApiEvent {
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
