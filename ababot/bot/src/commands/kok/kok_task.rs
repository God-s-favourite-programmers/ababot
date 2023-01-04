use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        command::CommandOptionType,
        interaction::{
            application_command::{ApplicationCommandInteraction, CommandDataOptionValue},
            InteractionResponseType,
        },
    },
    prelude::Context,
};

use super::{
    download::get,
    upload::{create_modal, save_small},
};

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) {
    for option in &command.data.options {
        if option.name == "get" {
            for sub_option in &option.options {
                if sub_option.name == "name" {
                    let name = sub_option.value.as_ref().unwrap().as_str().unwrap_or("kok");
                    get(ctx, command, name).await;
                    return;
                }
            }
        } else if option.name == "save" {
            for sub_option in &option.options {
                if sub_option.name == "small" {
                    let mut name = String::new();
                    for sub_sub_option in &sub_option.options {
                        if sub_sub_option.name == "name" {
                            name = sub_sub_option
                                .value
                                .as_ref()
                                .unwrap() // Handled by Discord that this is a string
                                .as_str()
                                .unwrap_or("kok")
                                .to_string();
                        } else if sub_sub_option.name == "file" {
                            if let CommandDataOptionValue::Attachment(file) =
                                sub_sub_option.resolved.as_ref().unwrap()
                            // Handled by Discord that this is a file
                            {
                                let file_bytes = if let Ok(file) = file.download().await {
                                    file
                                } else {
                                    tracing::warn!("Not able to download file");
                                    if let Err(why) = command
                                        .create_interaction_response(&ctx.http, |m| {
                                            m.kind(
                                                InteractionResponseType::ChannelMessageWithSource,
                                            )
                                            .interaction_response_data(|m| {
                                                m.content("Not able to download file")
                                            })
                                        })
                                        .await
                                    {
                                        tracing::warn!("Not able to respond to user: {:?}", why);
                                    }
                                    return;
                                };
                                save_small(ctx, command, name.as_str(), file_bytes).await;
                            }

                            return;
                        }
                    }
                } else if sub_option.name == "big" {
                    create_modal(ctx, command).await;
                    return;
                }
            }
        }
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("kok")
        .description("Kok")
        .create_option(|group| {
            group
                .name("save")
                .description("Add your contribution to the kok")
                .kind(CommandOptionType::SubCommandGroup)
                .create_sub_option(|sub| {
                    sub.name("small")
                        .description("Files smaller than 8MB")
                        .kind(CommandOptionType::SubCommand)
                        .create_sub_option(|opt| {
                            opt.name("name")
                                .description("Name of the kok")
                                .kind(CommandOptionType::String)
                                .required(true)
                        })
                        .create_sub_option(|opt| {
                            opt.name("file")
                                .description("File to save")
                                .kind(CommandOptionType::Attachment)
                                .required(true)
                        })
                })
                .create_sub_option(|sub| {
                    sub.name("big")
                        .description("Files bigger than 8MB. Limit 5GB")
                        .kind(CommandOptionType::SubCommand)
                })
        })
        .create_option(|option| {
            option
                .name("get")
                .description("Get a kok")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|opt| {
                    opt.name("name")
                        .description("Name of the kok")
                        .kind(CommandOptionType::String)
                        .required(true)
                })
        })
}
