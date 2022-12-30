use dateparser::parse;
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        command::CommandOptionType,
        interaction::{
            application_command::ApplicationCommandInteraction, InteractionResponseType,
        },
    },
    prelude::Context,
};

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) {
    let mut time = chrono::Duration::zero();

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
                    _ => {
                        panic!("Unknown sub option")
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
                            time = parsed_time.unwrap().date_naive().signed_duration_since(
                                chrono::Utc::now().naive_utc().date(),
                            );
                        } else {
                            panic!("Could not parse time");
                        }
                    }
                    _ => {
                        panic!("Unknown sub option")
                    }
                }
            }
        } else {
            panic!("Unknown option");
        }
    }
    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.content("Comming soon!"))
        })
        .await
        .unwrap();
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
