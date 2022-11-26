use std::{sync::Arc, time::Duration};

use chrono::{DateTime};

use serenity::{prelude::Context};


const START_TIME: (u8, u8, u8) = (7, 0, 0);
const URL: &str =
    "https://api.met.no/weatherapi/locationforecast/2.0/compact?lat=63.415398&lon=10.395053";

use crate::{background_tasks::yr::{types::{Root, Series}, image::create_image}, utils::{time::{Time, schedule, Interval, DAY_AS_SECONDS}, get_channel_id}};
pub async fn run(ctx: Arc<Context>) {
    let t = Time::new_unchecked(START_TIME.0, START_TIME.1, START_TIME.2).nearest_unchecked();
    schedule(
        Interval::EveryDeltaStartAt(Duration::from_secs(DAY_AS_SECONDS), t),
        || async { execute(ctx.clone()).await },
    )
    .await;
}

async fn execute(ctx: Arc<Context>) {
    tracing::info!("Fetching weather data from yr.no");
    let channel_id = match get_channel_id("weather", &ctx.http).await {
        Ok(id) => id,
        Err(e) => {
            tracing::error!("Failed to get channel id: {}", e);
            return;
        }
    };

    let weather_serie = match fetch_today_weather().await {
        Ok(w) => w,
        Err(e) => {
            tracing::warn!("Could not fetch weather. Reason: {}", e);
            return;
        }
    };

    let image_message = match create_image(weather_serie)
     {
        Ok(i) => i,
        Err(e) => {
            tracing::warn!("Could not create image. Reason: {}", e);
            return;
        }
     };

    image_message.save("weather.png").unwrap();

    let image_path = std::path::Path::new("weather.png");

    if let Err(e) = channel_id.send_files(&ctx.http, vec![image_path], |m| m).await {
        tracing::warn!("Could not send image. Reason: {}", e);
    }

    // Delete the image after it has been sent
    std::fs::remove_file(image_path).unwrap();

}

async fn fetch_today_weather() -> Result<Vec<Series>, String> {
    let client = reqwest::Client::new();
    let response = client
        .get(URL)
        .header("Accept", "application/json")
        .header("User-Agent", "DiscordBot")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;

    let mut weather = parse_weather(response)?;
    get_latest_weather(&mut weather)
}

fn parse_weather(response: String) -> Result<Vec<Series>, String> {
    let weather: Root = serde_json::from_str(&response).map_err(|_e| "Failed to parse json")?;
    Ok(weather.properties.timeseries)
}

fn get_latest_weather(weather: &mut [Series]) -> Result<Vec<Series>, String> {
    weather.sort_by(|a, b| {
        DateTime::parse_from_rfc3339(&a.time)
            .unwrap()
            .cmp(&DateTime::parse_from_rfc3339(&b.time).unwrap())
    });
    Ok(weather.to_vec())
}

// TODO: Incorporate this to the image
#[allow(dead_code)]
fn degrees_to_cardinal(degrees: f64) -> String {
    let directions = [
        "North",
        "North East",
        "East",
        "South East",
        "South",
        "South West",
        "West",
        "North West",
        "North",
    ];
    let index = ((degrees + 22.5) / 45.0) as usize;
    directions[index].to_string()
}
