use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::interaction::{
        application_command::ApplicationCommandInteraction, InteractionResponseType,
    },
    prelude::Context,
};
use tracing::instrument;

#[instrument(skip(ctx, command))]
pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) {
    let ans = "Come in".to_string();
    if let Err(why) = command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.content(ans))
        })
        .await
    {
        tracing::warn!("Failed to run command: {}", why);
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    tracing::debug!("Registering command knock");
    command.name("knock").description("Knock knock")
}
