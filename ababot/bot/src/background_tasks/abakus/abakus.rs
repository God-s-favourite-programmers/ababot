use std::sync::Arc;

use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use reqwest::Client;
use serenity::futures::future::join_all;
use serenity::{model::prelude::ChannelId, prelude::Context};
use tokio::time::sleep;

use crate::background_tasks::abakus::types::{ApiEvent, Event};
use crate::utils::{get_channel_id, schedule, Time, WEEK_AS_SECONDS};

const EVENT_URL: &str = "https://abakus.no/events/";
const REG_URL: &str = "https://lego.abakus.no/api/v1/events/";
pub async fn run(ctx: Arc<Context>) {
    //TODO: spawn another thread to watch for reactions to messages
    let today = chrono::offset::Local::now().date();
    let tomorrow = today.succ();

    schedule(
        Time::EveryDeltaStartAt(
            std::time::Duration::from_secs(WEEK_AS_SECONDS),
            // tomorrow.and_hms(8, 0, 0),
            today.and_hms(10,55, 0),
        ),
        || async { fetch_and_send(ctx.clone()).await },
    )
    .await
}

pub async fn fetch_and_send(ctx: Arc<Context>) {
    println!("Fetching events");
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

    let all_events = parse_events(fetched_data).await;

    let filtered_events = filter_existing_messages(ctx.clone(), all_events).await;

    for event in filtered_events {
        let channel_message = channel_id
            .send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.title(&event.title)
                        .description(&event.description)
                        .field(
                            "Registration",
                            &event
                                .reg_time
                                .map(|e| e.to_rfc2822().split('+').next().unwrap().to_owned())
                                .unwrap_or_else(|| "TBA".to_string()),
                            false,
                        )
                        .field(
                            "When",
                            &event.event_time.to_rfc2822().split('+').next().unwrap(),
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
            return;
        }

        if let Err(e) = channel_id
            .create_reaction(&ctx.http, channel_message.as_ref().unwrap(), '👍')
            .await
        {
            tracing::warn!("Could not create reaction. Reason: {}", e);
            return;
        }

        if let Err(e) = channel_id
            .create_reaction(&ctx.http, channel_message.as_ref().unwrap(), '👎')
            .await
        {
            tracing::warn!("Could not create reaction. Reason: {}", e);
            return;
        }

        if let Err(e) = channel_id
            .create_public_thread(&ctx.http, channel_message.as_ref().unwrap(), |f| {
                f.name(&event.title).auto_archive_duration(1440)
            })
            .await
        {
            tracing::warn!("Could not create thread. Reason: {}", e);
            return;
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

async fn parse_events(events: String) -> Vec<Event> {
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

    let mapped: Vec<Event> = events.into_iter().map(|e| e.into()).collect();
    let mut registration_times = Vec::with_capacity(mapped.len());

    // Clients to send
    let client = Client::new();
    let client_ref = Arc::new(client);

    for event in &mapped {
        let url = format!("{}{}", REG_URL, event.id);
        let event_time = get_reg_time(url, client_ref.clone());
        registration_times.push(event_time);
    }

    let time_awaited = join_all(registration_times).await;

    let mut actuall_res: Vec<Event> = Vec::with_capacity(mapped.len());

    for (mut event, time) in mapped.into_iter().zip(time_awaited.into_iter()) {
        match time {
            Ok(t) => {
                if t.date()
                    .signed_duration_since(chrono::Local::today())
                    .num_days()
                    == 0
                {
                    event.reg_time = Some(t);
                    actuall_res.push(event);
                }
            }
            Err(e) => {
                tracing::info!("Could not get reg time. Reason: {}", e);
            }
        }
    }
    // Return only events that have registration today
    actuall_res
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
        .flat_map(|m| m.embeds)
        .filter_map(|e| e.footer)
        .map(|f| f.text)
        .collect::<Vec<_>>();

    events
        .into_par_iter()
        .filter(|e| !embeds.contains(&e.id.to_string()))
        .collect()
}

async fn get_reg_time(
    url: String,
    client: Arc<Client>,
) -> Result<chrono::DateTime<chrono::Local>, Box<dyn std::error::Error + Send + Sync>> {
    let res = client.get(url).send().await?.text().await?;
    let v: serde_json::Value = serde_json::from_str(&res)?;
    let pools = v["pools"].as_array().ok_or("No results")?;
    if pools.is_empty() {
        return Err("No pools".into());
    }
    let time = pools[0]["activationDate"].as_str().ok_or("No time")?;
    Ok(chrono::DateTime::parse_from_rfc3339(time)?.with_timezone(&chrono::Local))
}
