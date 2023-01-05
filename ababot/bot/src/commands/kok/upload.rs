use serenity::{
    model::prelude::{
        component::{ActionRowComponent, InputTextStyle},
        interaction::{
            application_command::ApplicationCommandInteraction, modal::ModalSubmitInteraction,
            InteractionResponseType,
        },
    },
    prelude::Context,
};
use tokio::{fs::File, io::AsyncWriteExt};

pub async fn save_small(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    name: &str,
    bytes: Vec<u8>,
) {
    // Acknowledge user that file is being processed
    if let Err(why) = command
        .create_interaction_response(&ctx.http, |m| {
            m.kind(InteractionResponseType::DeferredChannelMessageWithSource)
        })
        .await
    {
        tracing::warn!("Not able to ack Modal: {:?}", why);
        return;
    }

    // File is small. Save the pdf
    let mut path = String::from("kok/");
    if name.ends_with(".pdf") {
        path.push_str(name);
    } else {
        path.push_str(name);
        path.push_str(".pdf");
    };
    let mut file = File::create(path).await.unwrap();
    file.write_all(&bytes).await.unwrap();

    // Respond to user
    if let Err(why) = command
        .create_followup_message(&ctx.http, |m| {
            m.content(format!("Saved file as {}.pdf", name))
        })
        .await
    {
        tracing::warn!("Not able to respond to user: {:?}", why);
    }

    // Save file
    file.sync_all().await.unwrap();
}

pub async fn save_big(ctx: &Context, command: &ModalSubmitInteraction) {
    if let Err(why) = command
        .create_interaction_response(&ctx.http, |m| {
            m.kind(InteractionResponseType::DeferredUpdateMessage)
        })
        .await
    {
        tracing::warn!("Not able to ack Modal: {:?}", why);
        return;
    }

    let string_answers = command
        .data
        .components
        .iter()
        .filter_map(|row| row.components.get(0))
        .map(|comp| match comp {
            ActionRowComponent::InputText(input) => input.value.clone(),
            _ => panic!("Not an input text"), // This means the modal is changed in a way that is not supported
        })
        .collect::<Vec<String>>();

    let (name, url) = match string_answers.get(0..2) {
        Some([name, url]) => (name, url),
        _ => {
            error(ctx, command).await;
            return;
        }
    };

    let page = match reqwest::get(url).await {
        Ok(page) => {
            if let Ok(page) = page.text().await {
                page
            } else {
                error(ctx, command).await;
                return;
            }
        }
        Err(_) => {
            error(ctx, command).await;
            return;
        }
    }; // page provided by user
    let download_url = if let Some(url) = local_parse(page) {
        url
    } else {
        error(ctx, command).await;
        return;
    }; // The actuall download url
    let download_file = match reqwest::get(&download_url).await {
        Ok(file) => {
            if let Ok(file) = file.bytes().await {
                file
            } else {
                error(ctx, command).await;
                return;
            }
        }
        Err(_) => {
            error(ctx, command).await;
            return;
        }
    }; // The actuall file

    // Create file
    let mut file = if let Ok(file) = File::create(format!("kok/{}.pdf", name)).await {
        file
    } else {
        error(ctx, command).await;
        return;
    };

    // Save file
    if let Err(why) = file.write_all(download_file.as_ref()).await {
        tracing::warn!("Not able to write to file: {:?}", why);
        return;
    }

    if let Err(why) = command
        .create_followup_message(&ctx.http, |m| {
            m.content(format!("Saved file as {}.pdf", name))
        })
        .await
    {
        tracing::warn!("Not able to ack Modal: {:?}", why);
    }
    file.sync_all().await.unwrap(); // Very unliklely to fail at this point. It's to ensure file is actually saved.
}

async fn error(ctx: &Context, command: &ModalSubmitInteraction) {
    if let Err(why) = command
        .create_followup_message(&ctx.http, |m| {
            m.content("Something went wrong. Please try again")
        })
        .await
    {
        tracing::warn!("Not able to complete kok: {:?}", why);
    }
}

pub fn local_parse(page: String) -> Option<String> {
    let document = scraper::Html::parse_document(&page);
    let selector = scraper::Selector::parse("#download-url").unwrap();
    document
        .select(&selector)
        .next()
        .and_then(|e| e.value().attr("href"))
        .map(|s| s.to_string())
}

// Interaction handling to this Modal is handled in lib.rs file in the interaction_create function
// before it is sent back down here to save_big function.
pub async fn create_modal(ctx: &Context, command: &ApplicationCommandInteraction) {
    match command
        .create_interaction_response(&ctx.http, |m| {
            m.kind(InteractionResponseType::Modal)
                .interaction_response_data(|d| {
                    d.content("Please select the file you want to download")
                        .custom_id("kok")
                        .title("Download")
                        .components(|c| {
                            c.create_action_row(|row| {
                                row.create_input_text(|menu| {
                                    menu.custom_id("name")
                                        .placeholder("Name")
                                        .label("Name")
                                        .style(InputTextStyle::Short)
                                })
                            })
                            .create_action_row(|row| {
                                row.create_input_text(|menu| {
                                    menu.custom_id("download")
                                        .placeholder("Download link")
                                        .label("Download link")
                                        .style(InputTextStyle::Short)
                                })
                            })
                        })
                })
        })
        .await
    {
        Ok(m) => m,
        Err(e) => {
            tracing::warn!("Not able to create modal: {:?}", e);
        }
    };
    if let Err(why) = command
        .create_followup_message(&ctx.http, |m| {
            m.content("Please wait while I save the file")
        })
        .await
    {
        tracing::warn!("Not able to respond to user: {:?}", why);
    }
}
