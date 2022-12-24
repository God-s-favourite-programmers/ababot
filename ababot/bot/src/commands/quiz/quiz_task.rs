use std::error::Error;

use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        command::CommandOptionType, interaction::application_command::ApplicationCommandInteraction,
    },
    prelude::Context,
};

use super::types::Quiz;

const API_URL: &str = "https://the-trivia-api.com/api/questions?";

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) {
    let mut url = String::from(API_URL);
    for option in &command.data.options {
        if option.name == "amount" {
            url.push_str(&format!("limit={}", option.value.as_ref().and_then(|v| v.as_i64()).unwrap_or(5)));
        }
    }
    let response = match fetch(&url).await {
        Ok(response) => response,
        Err(e) => {
            tracing::error!("Error fetching quiz: {}", e);
            return;
        }
    };
    let quiz = match parse(response) {
        Ok(quiz) => quiz,
        Err(e) => {
            tracing::error!("Error parsing quiz: {}", e);
            return;
        }
    };

}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    tracing::debug!("Registering command quiz");
    command
        .name("quiz")
        .description("When you need a quiz")
        .create_option(|option| {
            option
                .name("amount")
                .description("The number of questions in this quiz")
                .kind(CommandOptionType::Integer)
                .min_int_value(1)
                .max_int_value(20)
                .required(false)
        })
}

async fn fetch(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let response = reqwest::get(url).await?;
    let body = response.text().await?;
    Ok(body)
}

fn parse(response: String) -> Result<Quiz, Box<dyn Error>> {
    let quiz: Quiz = serde_json::from_str(&response)?;
    Ok(quiz)
}
