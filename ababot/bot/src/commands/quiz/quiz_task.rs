use std::{collections::HashMap, error::Error, time::Duration};

use rand::seq::SliceRandom;
use serenity::{
    builder::CreateApplicationCommand,
    futures::StreamExt,
    model::prelude::{
        command::CommandOptionType,
        interaction::{
            application_command::ApplicationCommandInteraction, InteractionResponseType,
        },
    },
    prelude::Context,
};

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
    // url.push_str("limit=5");
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

    let quiz_2 = quiz.clone();

    let channel_id = get_channel_id("quiz", &ctx.http).await.unwrap();

    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.content("Quiz time!"))
        })
        .await
        .unwrap();

    let mut question_string = String::new();
    for (i, question) in quiz.iter().enumerate() {
        question_string.push_str(&format!("{}. {}\n", i + 1, question.question));
    }
    let channel_message = channel_id
        .send_message(&ctx.http, |m| {
            m.content(question_string).components(|c| {
                quiz.into_iter().fold(c, |c, question| {
                    c.create_action_row(|row| {
                        row.create_select_menu(|menu| {
                            menu.custom_id(&question.id.to_string())
                                .placeholder("Select an answer");
                            menu.options(|f| {
                                let mut options: Vec<String> = question.incorrect_answers.clone();
                                options.push(question.correct_answer.clone());
                                options.shuffle(&mut rand::thread_rng());
                                options.into_iter().fold(f, |f, opt| {
                                    f.create_option(|o| o.label(opt.clone()).value(opt.clone()))
                                })
                            })
                        })
                    })
                })
            })
        })
        .await
        .unwrap();

    let mut collected_answers: HashMap<String, Vec<String>> = HashMap::new();
    let mut answer: HashMap<String, Vec<String>> = HashMap::new();

    let mut interaction_stream = channel_message
        .await_component_interactions(&ctx)
        .timeout(Duration::from_secs(40))
        .build();

    // TODO: Store values stored
    while let Some(interaction) = interaction_stream.next().await {
        let local_answer = interaction
            .data
            .values
            .get(0)
            .unwrap_or(&String::from("No answer"))
            .to_string();
        let who_answered = interaction.user.name.clone();
        collected_answers
            .entry(who_answered.clone())
            .or_insert_with(Vec::new)
            .push(local_answer);
        println!("{} answered", who_answered);
        interaction
            .create_interaction_response(&ctx, |r| {
                r.kind(InteractionResponseType::DeferredUpdateMessage)
            })
            .await
            .unwrap();
    }
    for question in quiz_2 {
        for (name, values) in &collected_answers {
            let filtered_list = values
                .iter()
                .filter(|&x| {
                    question.incorrect_answers.contains(x) || question.correct_answer == *x
                })
                .collect::<Vec<_>>();
            if filtered_list.last().unwrap_or(&&String::from("")) == &&question.correct_answer {
                answer.entry(name.clone()).or_insert_with(Vec::new).push("Correct".to_string());
            } else {
                answer.entry(name.clone()).or_insert_with(Vec::new).push("Incorrect".to_string());
            }
        }
    }

    println!("{:?}", answer);

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
