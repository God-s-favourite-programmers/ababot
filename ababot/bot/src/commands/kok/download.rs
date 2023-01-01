use std::error::Error;

use serenity::{
    model::prelude::interaction::{
        application_command::ApplicationCommandInteraction, InteractionResponseType,
    },
    prelude::Context,
};
use tokio::fs::read;

const URL: &str = "https://api.anonfiles.com/upload";

pub async fn get(ctx: &Context, command: &ApplicationCommandInteraction, file_str: &str) {
    let file = match read(file_str).await {
        Ok(file) => file,
        Err(_) => {
            error(ctx, command, "Invalid file id").await;
            return;
        }
    };
    // If file is smaller than 8MB, send it as an attachment
    if file.len() < 8_388_608 {
        match get_small(ctx, command, file_str).await {
            Ok(_) => return,
            Err(_) => {
                error(ctx, command, "Error sending file").await;
                return;
            }
        }
    }
    let form = reqwest::multipart::Form::new().part("file", reqwest::multipart::Part::bytes(file));
    let client = reqwest::Client::new();
    let response = match client.post(URL).multipart(form).send().await {
        Ok(response) => response,
        Err(_) => {
            error(ctx, command, "Error uploading file").await;
            return;
        }
    };
}

async fn get_small(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    file: &str,
) -> Result<(), String> {
    command
        .create_followup_message(&ctx.http, |m| {
            m.embed(|e| e.title("Kok").description("Kok").attachment(file))
        })
        .await
        .map_err(|_| String::from("Error sending file"))?;
    Ok(())
}

async fn success(ctx: &Context, command: &ApplicationCommandInteraction, url: &str) {
    command
        .create_interaction_response(&ctx.http, |m| {
            m.kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|m| m.content(url))
        })
        .await
        .unwrap();
}

async fn error(ctx: &Context, command: &ApplicationCommandInteraction, error: &str) {
    command
        .create_interaction_response(&ctx.http, |m| {
            m.kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|m| m.content(error).ephemeral(true))
        })
        .await
        .unwrap();
}
