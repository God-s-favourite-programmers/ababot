use std::{error::Error, time::Duration};

use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        command::CommandOptionType, interaction::{application_command::ApplicationCommandInteraction, InteractionResponseType}, component::ButtonStyle,
    },
    prelude::Context, futures::StreamExt,
};
use tokio::time::sleep;

use crate::utils::get_channel_id;

use super::types::Quiz;

const API_URL: &str = "https://the-trivia-api.com/api/questions?";

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) {
    let mut url = String::from(API_URL);
    for option in &command.data.options {
        if option.name == "amount" {
            url.push_str(&format!(
                "limit={}",
                option.value.as_ref().and_then(|v| v.as_i64()).unwrap_or(5)
            ));
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

    let channel_id = get_channel_id("quiz", &ctx.http).await.unwrap();

    command
        .create_interaction_response(&ctx.http, |response| {
            response.kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|message| message.content("Quiz time!"))
        })
        .await.unwrap();

    let channel_message = channel_id
        .send_message(&ctx.http, |m| {
            m.content("Quiz time!").components(|c| {
                c.create_action_row(|row| {
                    row.create_select_menu(|menu| {
                        menu.custom_id("Todays question")
                            .placeholder("Select an option");
                        menu.options(|f| {
                            f.create_option(|o| o.label("Option 1").value("1"))
                                .create_option(|o| o.label("Option 2").value("2"))
                                .create_option(|o| o.label("Option 3").value("3"))
                        })
                    })
                })
            })
        })
        .await
        .unwrap();

    println!("{:?}", quiz);

    let mut answer = 0;

    let mut interaction_stream = channel_message.await_component_interactions(&ctx).timeout(Duration::from_secs(20)).build();

    // TODO: Store values stored
    while let Some(interaction) = interaction_stream.next().await {
        answer = interaction.data.values[0].parse::<i64>().unwrap();
        interaction.create_interaction_response(&ctx, |r| {
            r.kind(InteractionResponseType::DeferredUpdateMessage)
        }).await.unwrap();
    }
    println!("Answer: {}", answer);

    channel_message.delete(&ctx.http).await.unwrap();
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
