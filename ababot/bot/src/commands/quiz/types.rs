use serde::{Deserialize, Serialize};
use serde_json::Value;

pub type Quiz = Vec<Question>;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Question {
    pub category: String,
    pub id: String,
    pub correct_answer: String,
    pub incorrect_answers: Vec<String>,
    pub question: String,
    pub tags: Vec<String>,
    #[serde(rename = "type")]
    pub type_field: String,
    pub difficulty: String,
    pub regions: Vec<Value>,
    pub is_niche: bool,
}
