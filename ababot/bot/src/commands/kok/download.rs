use std::path::{Path, PathBuf};

use serenity::{
    model::prelude::interaction::{
        application_command::ApplicationCommandInteraction, InteractionResponseType,
    },
    prelude::Context,
};
use tokio::fs::read;

use super::types::Annonfile;

const URL: &str = "https://api.anonfiles.com/upload";

pub async fn get(ctx: &Context, command: &ApplicationCommandInteraction, file_str: &str) {
    command
        .create_interaction_response(&ctx.http, |response| {
            response.kind(InteractionResponseType::DeferredChannelMessageWithSource)
        })
        .await
        .unwrap();

    let file_path = if file_str.ends_with(".pdf") {
        Path::new(file_str).to_owned()
    } else {
        Path::new(file_str)
            .with_extension("pdf")
            .as_path()
            .to_owned()
    };

    let file = match read(&file_path).await {
        Ok(file) => file,
        Err(_) => {
            error(ctx, command, "Invalid file id").await;
            return;
        }
    };
    // If file is smaller than 8MB, send it as an attachment
    if file.len() < 8_388_608 {
        match get_small(ctx, command, &file_path).await {
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
    let _parsed: Annonfile = serde_json::from_str(&response.text().await.unwrap()).unwrap();
}

async fn get_small(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    file: &PathBuf,
) -> Result<(), String> {
    let path = Path::new(file);

    command
        .create_followup_message(&ctx.http, |m| m.embed(|e| e.title("Kok")).add_file(path))
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
