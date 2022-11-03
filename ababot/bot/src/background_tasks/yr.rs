use std::{error::Error, sync::Arc};

use chrono::{DateTime, Utc, NaiveDateTime, Timelike};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use serenity::{model::prelude::ChannelId, prelude::Context};
use tokio::time::sleep;

const URL: &str =
    "https://api.met.no/weatherapi/locationforecast/2.0/compact?lat=63.415398&lon=10.395053";
const HEADER: &str = "Accept: application/json";

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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    #[serde(rename = "updated_at")]
    pub updated_at: String,
    pub units: Units,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Next12Hours {
    pub summary: Summary,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    loop {
        let now = chrono::Local::now();
        let mut target = chrono::Local::today().and_hms(11, 7, 15); // Time used for testing. Prod maybe 09:00?
        if now > target {
            target += chrono::Duration::days(1);
        }
        let duration = (target - now).to_std().unwrap();
        sleep(duration).await;

        let response = match fetch().await {
            Ok(r) => r,
            Err(e) => {
                println!("Error: {:?}", e);
                return;
            }
        };
        let weather = match parse_weather(response) {
            Ok(mut w) => get_latest_weather(&mut w),
            Err(e) => {
                println!("Error: {:?}", e);
                return;
            }
        };
        println!("{:?}", weather);

        let weather = match weather {
            Ok(w) => w,
            Err(_e) => {
                println!("No weather data");
                return;
            }
        };

        let time = DateTime::<Utc>::from_utc(
            NaiveDateTime::parse_from_str(&weather.time, "%Y-%m-%dT%H:%M:%SZ").unwrap(),
            Utc,
        ).format("%d/%m at %H:%M").to_string();

        let message = ChannelId(772092284153757719)
            .send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.title("Weather")
                        .field("Time", time, false)
                        .field(
                            "Air temperature",
                            weather.data.instant.details.air_temperature,
                            false,
                        )
                        .field(
                            "Cloud area fraction",
                            weather.data.instant.details.cloud_area_fraction,
                            false,
                        )
                        .field(
                            "Relative humidity",
                            weather.data.instant.details.relative_humidity,
                            false,
                        )
                        .field(
                            "Wind from direction",
                            weather.data.instant.details.wind_from_direction,
                            false,
                        )
                        .field("Wind speed", weather.data.instant.details.wind_speed, false)
                })
            })
            .await;
        if let Err(e) = message {
            println!("Error: {:?}", e);
        }
    }
}

async fn fetch() -> Result<String, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let response = client
        .get(URL)
        .header("Accept", "application/json")
        .header("User-Agent", "DiscordBot")
        .send()
        .await?
        .text()
        .await?;
    Ok(response)
}

fn parse_weather(response: String) -> Result<Vec<Series>, Box<dyn Error>> {
    let weather: Root = serde_json::from_str(&response)?;
    Ok(weather.properties.timeseries)
}

fn get_latest_weather(weather: &mut Vec<Series>) -> Result<Series, Box<dyn Error>> {
    weather.sort_by(|a, b| {
        DateTime::parse_from_rfc3339(&a.time)
            .unwrap()
            .cmp(&DateTime::parse_from_rfc3339(&b.time).unwrap())
    });
    let serie_at_12 = weather.into_iter().find(|s| {
        DateTime::parse_from_rfc3339(&s.time)
            .unwrap()
            .hour()
            == 12
    });

    if let Some(s) = serie_at_12 {
        return Ok(s.clone());
    } else {
        return Err("No weather at 12".into());
    }
}
