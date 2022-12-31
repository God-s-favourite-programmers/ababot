use std::sync::Arc;

use dateparser::parse;
use serenity::{
    builder::CreateApplicationCommand,
    model::{
        prelude::{
            command::CommandOptionType,
            interaction::{
                application_command::ApplicationCommandInteraction, InteractionResponseType,
            },
        },
        user::User,
    },
    prelude::Context,
};

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) {
    let mut time = chrono::Duration::zero();
    let mut message = String::new();
    let user = Arc::new(&command.user);

    for option in &command.data.options {
        if option.name == "form" {
            for sub_option in &option.options {
                match sub_option.name.as_str() {
                    "day" => {
                        time = time
                            + chrono::Duration::days(
                                sub_option.value.as_ref().unwrap().as_i64().unwrap_or(0),
                            );
                    }
                    "hour" => {
                        time = time
                            + chrono::Duration::hours(
                                sub_option.value.as_ref().unwrap().as_i64().unwrap_or(0),
                            );
                    }
                    "minute" => {
                        time = time
                            + chrono::Duration::minutes(
                                sub_option.value.as_ref().unwrap().as_i64().unwrap_or(0),
                            );
                    }
                    "message" => {
                        message = sub_option
                            .value
                            .as_ref()
                            .unwrap()
                            .as_str()
                            .unwrap_or("")
                            .to_string();
                    }
                    _ => {
                        println!("Unknown option 1 {}", sub_option.name);
                    }
                }
            }
        } else if option.name == "string" {
            for sub_option in &option.options {
                match sub_option.name.as_str() {
                    "time" => {
                        let parsed_time =
                            parse(sub_option.value.as_ref().unwrap().as_str().unwrap_or("0"));
                        if parsed_time.is_ok() {
                            time = parsed_time
                                .unwrap()
                                .signed_duration_since(chrono::Utc::now())
                        } else {
                            if let Err(why) = command
                                .create_interaction_response(&ctx.http, |response| {
                                    response
                                        .kind(InteractionResponseType::ChannelMessageWithSource)
                                        .interaction_response_data(|m| {
                                            m.content(
                                                format!(
                                                    "I was not able to parse time\n
                                                     {}",
                                                    parsed_time.err().unwrap()
                                                )
                                                .as_str(),
                                            )
                                        })
                                })
                                .await
                            {
                                tracing::warn!("Failed to send message: {:?}", why);
                            }
                            return;
                        }
                    }
                    "message" => {
                        message = sub_option
                            .value
                            .as_ref()
                            .unwrap()
                            .as_str()
                            .unwrap_or("")
                            .to_string();
                    }
                    _ => {
                        println!("Unknown option 2 {}", sub_option.name);
                    }
                }
            }
        } else {
            tracing::warn!("Unknown option {}", option.name);
        }
    }
    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|m| {
                    m.content(
                        format!(
                            "Not implemented, but should sleep for {} days, {} hours and {} minutes \nMessage: {}",
                            time.num_days(),
                            time.num_hours() % 24,
                            time.num_minutes() % 60,
                            message
                        )
                        .as_str(),
                    )
                })
        })
        .await
        .unwrap();

    sleep_and_remind(time, message, ctx, &user);
}

fn sleep_and_remind(time: chrono::Duration, message: String, ctx: &Context, user: &User) {
    let ctx = ctx.clone();
    let user = user.clone();
    tokio::spawn(async move {
        tokio::time::sleep(time.to_std().unwrap()).await;
        // Send DM to user
        if let Err(why) = user
            .direct_message(&ctx.http, |m| m.content(message.as_str()))
            .await
        {
            tracing::warn!("Failed to send message: {:?}", why);
        }
    });
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    tracing::debug!("Registering command remindme");
    command
        .name("remindme")
        .description("Reminds you of something")
        .create_option(|option| {
            option
                .name("form")
                .description("Just fill in the fields")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|opt| {
                    opt.name("message")
                        .description("The message to remind you of")
                        .kind(CommandOptionType::String)
                        .required(true)
                })
                .create_sub_option(|opt| {
                    opt.name("day")
                        .description("Days from now")
                        .kind(CommandOptionType::Integer)
                })
                .create_sub_option(|opt| {
                    opt.name("hour")
                        .description("Hours from now")
                        .kind(CommandOptionType::Integer)
                })
                .create_sub_option(|opt| {
                    opt.name("minute")
                        .description("Minutes from now")
                        .kind(CommandOptionType::Integer)
                })
        })
        .create_option(|option| {
            option
                .name("string")
                .description("Parse time from string")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|opt| {
                    opt.name("message")
                        .description("The message to remind you of")
                        .kind(CommandOptionType::String)
                        .required(true)
                })
                .create_sub_option(|opt| {
                    opt.name("time")
                        .description("The time to remind you of")
                        .kind(CommandOptionType::String)
                        .required(true)
                })
        })
}
