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
    let mut min = 0;
    for option in &command.data.options {
        if option.name == "group" {
            for subopt in &option.options {
                if subopt.name == "intger" {
                    for subsubopt in &subopt.options {
                        if subsubopt.name == "times" {
                            min = subsubopt
                                .value
                                .as_ref()
                                .and_then(|v| v.as_i64())
                                .unwrap_or(0);
                        }
                    }
                }
            }
        }
    }
    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.content(format!(
                        "This command is not yet implemented but here is the value you gave me: {}",
                        min
                    ))
                })
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
                .name("group")
                .description("The group to remind you of")
                .kind(CommandOptionType::SubCommandGroup)
                .create_sub_option(|subopt| {
                    subopt
                        .name("intger")
                        .description("Reminds you of an integer")
                        .kind(CommandOptionType::SubCommand)
                        .create_sub_option(|opt| {
                            opt.name("times")
                                .description("The time to remind you of")
                                .kind(CommandOptionType::Integer)
                                .required(true)
                        })
                })
            // .create_sub_option(|subopt| {
            //     subopt
            //         .name("string")
            //         .description("Reminds you of a string")
            //         .kind(CommandOptionType::SubCommand)
            //         .create_sub_option(|opt| {
            //             opt.name("time")
            //                 .description("The time to remind you of")
            //                 .kind(CommandOptionType::String)
            //         })
            // })
        })
        .create_option(|option| {
            option
                .name("message")
                .description("The message to remind you of")
                .kind(CommandOptionType::SubCommandGroup)
                .create_sub_option(|subopt| {
                    subopt
                        .name("string")
                        .description("Reminds you of a string")
                        .kind(CommandOptionType::SubCommand)
                        .create_sub_option(|opt| {
                            opt.name("message")
                                .description("The message to remind you of")
                                .kind(CommandOptionType::String)
                        })
                })
            // .create_sub_option(|subopt| {
            //     subopt
            //         .name("integer")
            //         .description("Reminds you of an integer")
            //         .kind(CommandOptionType::SubCommand)
            //         .create_sub_option(|opt| {
            //             opt.name("message")
            //                 .description("The message to remind you of")
            //                 .kind(CommandOptionType::Integer)
            //         })
            // })
        })
}
