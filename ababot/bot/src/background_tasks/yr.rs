use std::sync::Arc;

use serenity::{model::prelude::ChannelId, prelude::Context};


use serde::{Deserialize, Serialize};

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
    let _noe = ChannelId(772092284153757719)
        .send_message(&ctx.http, |m| m.embed(|e| e.title("Asyncly doing stuff 2")))
        .await;
    if let Err(e) = _noe {
        // TODO: log
        println!("Error: {:?}", e);
    }
}
