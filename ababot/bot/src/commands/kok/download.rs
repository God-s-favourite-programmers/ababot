use std::path::{Path, PathBuf};

use reqwest::multipart::{Form, Part};
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
    if let Err(why) = command
        .create_interaction_response(&ctx.http, |response| {
            response.kind(InteractionResponseType::DeferredChannelMessageWithSource)
        })
        .await
    {
        tracing::warn!("Not able to ack interaction: {:?}", why);
        return;
    }

    if file_str.is_empty() || file_str.contains("/") {
        error(ctx, command, "Illegal file type").await;
        return;
    }

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
        match get_small(
            ctx,
            command,
            &file_path,
            file_path.to_str().unwrap().to_string(),
            // Allowed unwrap() because file_path is properly handled in saving process
        )
        .await
        {
            Ok(_) => return,
            Err(_) => {
                error(ctx, command, "Error sending file").await;
                return;
            }
        }
    } else {
        tracing::debug!("Handling big file");
        match get_big(ctx, command, file, file_path.to_str().unwrap().to_string()).await {
            // Allowed unwrap() because file_path is properly handled in saving process
            Ok(_) => return,
            Err(_) => {
                error(ctx, command, "Error uploading file").await;
                return;
            }
        }
    }
}

async fn get_small(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    file: &PathBuf,
    name: String,
) -> Result<(), String> {
    let path = Path::new(file);

    command
        .create_followup_message(&ctx.http, |m| m.embed(|e| e.title(name)).add_file(path))
        .await
        .map_err(|_| String::from("Error sending file"))?;
    Ok(())
}

async fn get_big(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    file: Vec<u8>,
    name: String,
) -> Result<(), String> {
    let file_part = Part::bytes(file)
        .file_name(name.clone())
        .mime_str("application/pdf")
        .unwrap(); // In the saving part I have forced it to be a pdf so unwrap is ok. Garbage in, garbage out

    let form = Form::new().part("file", file_part);
    let client = reqwest::Client::new();
    let response = match client.post(URL).multipart(form).send().await {
        Ok(response) => response,
        Err(_) => {
            error(ctx, command, "Error uploading file").await;
            return Err(String::from("Error uploading file"));
        }
    };

    // Respone from Annonfile
    let parsed: Annonfile = match serde_json::from_str(&response.text().await.unwrap()) {
        // I have assumed that some text will be there to be parsed. Unwrap ok above
        Ok(parsed) => parsed,
        Err(_) => {
            error(ctx, command, "Error uploading file").await;
            return Err(String::from("Error uploading file"));
        }
    };

    command
        .create_followup_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title(name).url(parsed.data.file.url.full).field(
                    "May not be valid after",
                    "10 days", // TODO Get ten days from now
                    false,
                )
            })
        })
        .await
        .map_err(|_| String::from("Error sending file"))?;

    Ok(())
}

async fn error(ctx: &Context, command: &ApplicationCommandInteraction, error: &str) {
    if let Err(why) = command
        .create_followup_message(&ctx.http, |m| {
            m.embed(|e| e.title("Kok").description(error))
        })
        .await
    {
        tracing::warn!("Not able to send error message: {:?}", why);
    }
}
