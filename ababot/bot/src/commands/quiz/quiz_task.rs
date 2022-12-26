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

const API_URL: &str = "https://the-trivia-api.com/api/questions?limit=5";

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) {
    let response = match fetch(API_URL).await {
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

    let channel_id = match get_channel_id("quiz", &ctx.http).await {
        Ok(channel_id) => channel_id,
        Err(e) => {
            tracing::error!("Error getting channel id: {}", e);
            return;
        }
    };

    if let Err(why) = command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.content("Quiz time!"))
        })
        .await
    {
        tracing::warn!("Failed to run command: {}", why);
        return;
    }

    let mut question_string = String::new();
    for (i, question) in quiz.iter().enumerate() {
        question_string.push_str(&format!("{}. {}\n", i + 1, question.question));
    }
    let channel_message = match channel_id
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
    {
        Ok(m) => m,
        Err(e) => {
            tracing::error!("Error sending quiz: {}", e);
            return;
        }
    };

    let mut collected_answers: HashMap<String, Vec<String>> = HashMap::new();
    let mut answer: HashMap<String, Vec<String>> = HashMap::new();

    let mut quiz_time = 1;
    for option in &command.data.options {
        if option.name == "time" {
            quiz_time = option.value.as_ref().and_then(|v| v.as_u64()).unwrap_or(1);
        }
    }

    let mut interaction_stream = channel_message
        .await_component_interactions(ctx)
        .timeout(Duration::from_secs(quiz_time * 60))
        .build();

    while let Some(interaction) = interaction_stream.next().await {
        let local_answer = interaction
            .data
            .values
            .get(0)
            .unwrap_or(&String::from("No answer"))
            .to_string();

        let who_answered = interaction
            .member
            .as_ref()
            .unwrap()
            .nick
            .clone()
            .unwrap_or_else(|| interaction.user.name.clone());

        collected_answers
            .entry(who_answered.clone())
            .or_insert_with(Vec::new)
            .push(local_answer);

        if let Err(why) = interaction
            .create_interaction_response(&ctx, |r| {
                r.kind(InteractionResponseType::DeferredUpdateMessage)
            })
            .await
        {
            tracing::error!("Error responding to interaction: {:?}", why);
        }
    }
    for question in &quiz_2 {
        for (name, values) in &collected_answers {
            let filtered_list = values
                .iter()
                .filter(|&x| {
                    question.incorrect_answers.contains(x) || question.correct_answer == *x
                })
                .collect::<Vec<_>>();
            if filtered_list.last().unwrap_or(&&String::from("")) == &&question.correct_answer {
                answer
                    .entry(name.clone())
                    .or_insert_with(Vec::new)
                    .push("🟢".to_string());
            } else {
                answer
                    .entry(name.clone())
                    .or_insert_with(Vec::new)
                    .push("🔴".to_string());
            }
        }
    }

    if let Err(why) = channel_message.delete(&ctx.http).await {
        tracing::error!("Error deleting quiz: {:?}", why);
    }

    if let Err(why) = channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                quiz_2.iter().fold(e, |e, question| {
                    e.field(
                        question.question.clone(),
                        question.correct_answer.clone(),
                        false,
                    )
                    .title("Answers")
                })
            })
        })
        .await
    {
        tracing::error!("Error sending answers: {:?}", why);
    }

    if let Err(why) = channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                answer.keys().fold(e, |e, k| {
                    let user_answers = answer.get(k).unwrap();
                    let result_string = user_answers.join(" ");
                    e.field(k, result_string, false).title("Results")
                })
            })
        })
        .await
    {
        tracing::error!("Error sending results: {:?}", why);
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    tracing::debug!("Registering command quiz");
    command
        .name("quiz")
        .description("Answer these fiver questions in a limited amount of time")
        .create_option(|opt| {
            opt.name("time")
                .description("Amount of time for the quiz")
                .kind(CommandOptionType::Integer)
                .min_int_value(1)
                .max_int_value(5)
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
