use std::path::Path;

use reqwest::multipart::{Form, Part};
use serenity::{
    model::prelude::interaction::{
        application_command::ApplicationCommandInteraction, InteractionResponseType,
    },
    prelude::Context,
};
use tokio::fs::read;

use super::{types::Annonfile, upload::local_parse};

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

    if file_str.is_empty() || file_str.contains('/') {
        error(ctx, command, "Illegal path").await;
        return;
    }

    let mut base = String::from("kok/");
    if file_str.ends_with(".pdf") {
        base.push_str(file_str);
    } else {
        base.push_str(&format!("{}.pdf", file_str));
    }

    let file_path = Path::new(&base);

    let file = match read(&file_path).await {
        Ok(file) => file,
        Err(_) => {
            error(ctx, command, "Invalid file id").await;
            return;
        }
    };
    // If file is smaller than 8MB, send it as an attachment
    if file.len() < 8_388_608 {
        match get_small(ctx, command, file_path).await {
            Ok(_) => (),
            Err(_) => {
                error(ctx, command, "Error sending file").await;
            }
        }
    } else {
        tracing::debug!("Handling big file");
        match get_big(ctx, command, file, base).await {
            // Allowed unwrap() because file_path is properly handled in saving process
            Ok(_) => (),
            Err(_) => {
                error(ctx, command, "Error uploading file").await;
            }
        }
    }
}

async fn get_small(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    file: &Path,
) -> Result<(), String> {
    command
        .create_followup_message(&ctx.http, |m| m.add_file(file))
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
    let parsed: Annonfile = match serde_json::from_str(
        &response
            .text()
            .await
            .unwrap_or_else(|_| "no response".to_string()),
    ) {
        // I have assumed that some text will be there to be parsed. Unwrap ok above
        Ok(parsed) => parsed,
        Err(_) => {
            error(ctx, command, "Error uploading file").await;
            return Err(String::from("Error uploading file"));
        }
    };

    let page = match reqwest::get(parsed.data.file.url.full).await {
        Ok(page) => match page.text().await {
            Ok(page) => page,
            Err(_) => {
                return Err(String::from("Error uploading file"));
            }
        },
        Err(_) => {
            return Err(String::from("Error uploading file"));
        }
    };
    let url = if let Some(url) = local_parse(page) {
        url
    } else {
        return Err(String::from("Error uploading file"));
    };

    command
        .create_followup_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title(name.split('/').nth(1).unwrap_or("Kok"))
                    .url(url)
                    .field(
                        "May not be valid after",
                        chrono::Utc::now()
                            .checked_add_signed(chrono::Duration::days(7))
                            .unwrap()
                            .format("%d/%m"),
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
        .create_followup_message(&ctx.http, |m| m.content(format!("Error: {}", error)))
        .await
    {
        tracing::warn!("Not able to send error message: {:?}", why);
    }
}
