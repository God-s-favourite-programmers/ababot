use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiEvent {
    pub title: Option<String>,
    pub description: Option<String>,
    pub event_time: Option<String>,
    pub event_location: Option<String>,
    pub thumbnail: Option<String>,
}
#[derive(Debug)]
pub struct Event {
    pub title: String,
    pub description: String,
    pub event_time: DateTime<Utc>,
    pub event_location: String,
    pub thumbnail: String,
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
