use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        command::CommandOptionType, interaction::application_command::ApplicationCommandInteraction,
    },
    prelude::Context,
};

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) {
    let channel_id = command.channel_id;
    channel_id
        .say(&ctx.http, "Not yet implemented :)")
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
                .name("time")
                .description("The time to remind you")
                .required(true)
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|opt| {
                    opt.name("string")
                        .description("The time to remind you")
                        .required(false)
                        .kind(CommandOptionType::String)
                })
                .create_sub_option(|opt| {
                    opt.name("number")
                        .description("The time to remind you")
                        .required(false)
                        .kind(CommandOptionType::Integer)
                })
        })
}
