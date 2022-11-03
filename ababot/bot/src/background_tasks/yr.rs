use std::sync::Arc;

use chrono::{DateTime, NaiveDateTime, Timelike, Utc, Local};
use serde::{Deserialize, Serialize};
use serenity::{model::prelude::ChannelId, prelude::Context};

use crate::utils::{schedule, Time};

const URL: &str =
    "https://api.met.no/weatherapi/locationforecast/2.0/compact?lat=63.415398&lon=10.395053";

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    #[serde(rename = "type")]
    pub type_field: String,
    pub geometry: Geometry,
    pub properties: Properties,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Geometry {
    #[serde(rename = "type")]
    pub type_field: String,
    pub coordinates: Vec<f64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Properties {
    pub meta: Meta,
    pub timeseries: Vec<Series>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    #[serde(rename = "updated_at")]
    pub updated_at: String,
    pub units: Units,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Units {
    #[serde(rename = "air_pressure_at_sea_level")]
    pub air_pressure_at_sea_level: String,
    #[serde(rename = "air_temperature")]
    pub air_temperature: String,
    #[serde(rename = "cloud_area_fraction")]
    pub cloud_area_fraction: String,
    #[serde(rename = "precipitation_amount")]
    pub precipitation_amount: String,
    #[serde(rename = "relative_humidity")]
    pub relative_humidity: String,
    #[serde(rename = "wind_from_direction")]
    pub wind_from_direction: String,
    #[serde(rename = "wind_speed")]
    pub wind_speed: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Series {
    pub time: String,
    pub data: Data,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Data {
    pub instant: Instant,
    #[serde(rename = "next_12_hours")]
    pub next_12_hours: Option<Next12Hours>,
    #[serde(rename = "next_1_hours")]
    pub next_1_hours: Option<Next1Hours>,
    #[serde(rename = "next_6_hours")]
    pub next_6_hours: Option<Next6Hours>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Instant {
    pub details: Details,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Details {
    #[serde(rename = "air_pressure_at_sea_level")]
    pub air_pressure_at_sea_level: f64,
    #[serde(rename = "air_temperature")]
    pub air_temperature: f64,
    #[serde(rename = "cloud_area_fraction")]
    pub cloud_area_fraction: f64,
    #[serde(rename = "relative_humidity")]
    pub relative_humidity: f64,
    #[serde(rename = "wind_from_direction")]
    pub wind_from_direction: f64,
    #[serde(rename = "wind_speed")]
    pub wind_speed: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Next12Hours {
    pub summary: Summary,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    #[serde(rename = "symbol_code")]
    pub symbol_code: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Next1Hours {
    pub summary: Summary2,
    pub details: Details2,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary2 {
    #[serde(rename = "symbol_code")]
    pub symbol_code: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Details2 {
    #[serde(rename = "precipitation_amount")]
    pub precipitation_amount: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Next6Hours {
    pub summary: Summary3,
    pub details: Details3,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary3 {
    #[serde(rename = "symbol_code")]
    pub symbol_code: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Details3 {
    #[serde(rename = "precipitation_amount")]
    pub precipitation_amount: f64,
}

pub async fn run(ctx: Arc<Context>) {
    schedule(Time::EveryTime(Local::now().date().and_hms(8, 0, 0)), || async {
        execute(ctx.clone()).await
    }).await;
}

async fn execute(ctx: Arc<Context>) {
        let weather_serie = match fetch_today_weather().await {
            Ok(w) => w,
            Err(_e) => {
                println!("No weather data"); // TODO: Log error
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
            println!("Error: {:?}", e);
        }
}

async fn fetch_today_weather() -> Result<Series, String> {
    let client = reqwest::Client::new();
    let response = client
        .get(URL)
        .header("Accept", "application/json")
        .header("User-Agent", "DiscordBot")
        .send()
        .await.map_err(|e| e.to_string())?
        .text()
        .await.map_err(|e| e.to_string())?;

    let weather = parse_weather(response)?;
    Ok(get_latest_weather(&mut weather.clone())?)
}

fn parse_weather(response: String) -> Result<Vec<Series>, String> {
    let weather: Root = serde_json::from_str(&response).map_err(|_e| "Failed to parse json")?;
    Ok(weather.properties.timeseries)
}

fn get_latest_weather(weather: &mut Vec<Series>) -> Result<Series, String> {
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
    let directions = ["North", "North East", "East", "South East", "South", "South West", "West", "North West", "North"];
    let index = ((degrees + 22.5) / 45.0) as usize;
    directions[index].to_string()
}
