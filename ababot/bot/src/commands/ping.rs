use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::interaction::{
        application_command::{ApplicationCommandInteraction, CommandDataOption},
        InteractionResponseType,
    },
    prelude::Context,
};

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) {
    let pong = "Pong!".to_string();
    if let Err(why) = command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.content(pong))
        })
        .await
    {
        tracing::warn!("Failed to run command: {}", why);
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("ping").description("A ping command")
}
