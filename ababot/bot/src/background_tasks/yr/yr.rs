use std::sync::Arc;

use chrono::{DateTime, Local, NaiveDateTime, Timelike, Utc};
use serenity::{model::prelude::ChannelId, prelude::Context};

use crate::utils::{schedule, Time};

const URL: &str =
    "https://api.met.no/weatherapi/locationforecast/2.0/compact?lat=63.415398&lon=10.395053";

use crate::background_tasks::yr::types::{Root, Series};
pub async fn run(ctx: Arc<Context>) {
    schedule(
        Time::EveryTime(Local::now().date().and_hms(8, 0, 0)),
        || async { execute(ctx.clone()).await },
    )
    .await;
}

async fn execute(ctx: Arc<Context>) {
    let weather_serie = match fetch_today_weather().await {
        Ok(w) => w,
        Err(e) => {
            tracing::warn!("Could not fetch weather. Reason: {}", e);
            return;
        }
    };

    let time = DateTime::<Utc>::from_utc(
        NaiveDateTime::parse_from_str(&weather_serie.time, "%Y-%m-%dT%H:%M:%SZ").unwrap(),
        Utc,
    )
    .format("%d/%m at %H:%M")
    .to_string();

    let message = ChannelId(772092284153757719)
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title("Weather Report")
                    .field("Time", time, false)
                    .field(
                        "Air temperature",
                        weather_serie.data.instant.details.air_temperature,
                        false,
                    )
                    .field(
                        "Cloud area fraction",
                        weather_serie.data.instant.details.cloud_area_fraction,
                        false,
                    )
                    .field(
                        "Relative humidity",
                        weather_serie.data.instant.details.relative_humidity,
                        false,
                    )
                    .field(
                        "Wind from direction",
                        degrees_to_cardinal(weather_serie.data.instant.details.wind_from_direction),
                        false,
                    )
                    .field(
                        "Wind speed",
                        weather_serie.data.instant.details.wind_speed,
                        false,
                    )
            })
        })
        .await;
    if let Err(e) = message {
        tracing::warn!("Could not send weather report to Discord. Reason: {}", e);
    }
}

async fn fetch_today_weather() -> Result<Series, String> {
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
    Ok(get_latest_weather(&mut weather)?)
}

fn parse_weather(response: String) -> Result<Vec<Series>, String> {
    let weather: Root = serde_json::from_str(&response).map_err(|_e| "Failed to parse json")?;
    Ok(weather.properties.timeseries)
}

fn get_latest_weather(weather: &mut [Series]) -> Result<Series, String> {
    weather.sort_by(|a, b| {
        DateTime::parse_from_rfc3339(&a.time)
            .unwrap()
            .cmp(&DateTime::parse_from_rfc3339(&b.time).unwrap())
    });
    let serie_at_12 = weather
        .iter_mut()
        .find(|s| DateTime::parse_from_rfc3339(&s.time).unwrap().hour() == 12);

    if let Some(s) = serie_at_12 {
        Ok(s.clone())
    } else {
        Err("No weather at 12".into())
    }
}

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
