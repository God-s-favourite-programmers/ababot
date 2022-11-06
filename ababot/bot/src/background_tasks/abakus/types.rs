use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiEvent {
    pub id: i32,
    pub title: Option<String>,
    pub description: Option<String>,
    pub event_time: Option<String>,
    pub event_location: Option<String>,
    pub thumbnail: Option<String>,
}
#[derive(Debug)]
pub struct Event {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub event_time: DateTime<Local>,
    pub event_location: String,
    pub thumbnail: String,
    pub reg_time: Option<DateTime<Local>>,
}

impl From<ApiEvent> for Event {
    fn from(api_event: ApiEvent) -> Self {
        Event {
            id: api_event.id,
            title: api_event.title.unwrap_or_else(|| "No title".to_string()),
            description: api_event
                .description
                .unwrap_or_else(|| "No description".to_string()),
            event_time: api_event
                .event_time
                .unwrap_or_else(|| "".to_string())
                .parse()
                .unwrap_or_else(|_| Local::now()),
            event_location: api_event
                .event_location
                .unwrap_or_else(|| "N/A".to_string()),
            thumbnail: api_event.thumbnail.unwrap_or_else(|| "N/A".to_string()),
            reg_time: None,
        }
    }
}
