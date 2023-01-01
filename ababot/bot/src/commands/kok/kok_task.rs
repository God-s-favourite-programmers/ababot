use reqwest::multipart;
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

use super::download::get;


pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) {
    for option in &command.data.options {
        if option.name == "get" {
            for sub_option in &option.options {
                if sub_option.name == "name" {
                    let name = sub_option.value.as_ref().unwrap().as_str().unwrap();
                    get(ctx, command, name).await;
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
                        .description("Small kok")
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
                        .description("Big kok")
                        .kind(CommandOptionType::SubCommand)
                        .create_sub_option(|opt| {
                            opt.name("name")
                                .description("Name of the kok")
                                .kind(CommandOptionType::String)
                                .required(true)
                        })
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
