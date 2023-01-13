use std::env;

use serenity::{
    futures::StreamExt,
    model::{
        prelude::{
            component::{ActionRowComponent, InputTextStyle},
            interaction::{
                application_command::ApplicationCommandInteraction, modal::ModalSubmitInteraction,
                InteractionResponseType,
            },
            GuildId,
        },
        user::User,
    },
    prelude::Context,
};
use tokio::{fs::File, io::AsyncWriteExt};

use crate::utils::get_channel_id;

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
    let mut path = String::from("/kok/");
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
    update_kok_catalogue(ctx, name.to_string(), &command.user).await;
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
    let mut file = if let Ok(file) = File::create(format!("/kok/{}.pdf", name)).await {
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

    update_kok_catalogue(ctx, name.to_string(), &command.user).await;
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

async fn update_kok_catalogue(ctx: &Context, name: String, user: &User) {
    let channel_id = if let Ok(channel) = get_channel_id("suppekjøkkenet", &ctx.http).await {
        channel
    } else {
        tracing::warn!("Not able to get channel id");
        return;
    };

    let mut bot_messages = Vec::with_capacity(100);
    let mut messages = channel_id.messages_iter(&ctx.http).boxed();
    while let Some(message) = messages.next().await {
        let message = match message {
            Ok(message) => message,
            Err(why) => {
                tracing::warn!("Not able to get message: {:?}", why);
                continue;
            }
        };
        if message.author.id == ctx.cache.current_user_id() {
            bot_messages.push(message);
        }
    }
    bot_messages = bot_messages
        .into_iter()
        .filter_map(|m| {
            if let Some(embed) = m.embeds.get(0) {
                if embed.title.as_ref().unwrap_or(&"".to_string()) == &name {
                    Some(m)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();
    // Find first
    let message_id = bot_messages.get(0).map(|m| m.id);
    let mut updated = false;
    if let Some(message_id) = message_id {
        match channel_id.delete_message(&ctx.http, message_id).await {
            Ok(_) => updated = true,
            Err(why) => tracing::warn!("Not able to delete message: {:?}", why),
        }
    }

    let guild_id = GuildId(
        env::var("GUILD_ID")
            .expect("Expected GUILD_ID in environment")
            .parse()
            .expect("GUILD_ID must be an integer"),
    ); // Maybe store guildID in the cache?

    let user_name = user
        .nick_in(&ctx.http, guild_id)
        .await
        .unwrap_or_else(|| user.name.clone());

    // Remove .pdf from name if it exists
    let name = if name.ends_with(".pdf") {
        name[..name.len() - 4].to_string()
    } else {
        name
    };

    if let Err(why) = channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title(name)
                    .description(if updated {
                        format!("{} has updated the file", user_name)
                    } else {
                        format!("{} has added a new file", user_name)
                    })
                    .color(0x00ff00)
            })
        })
        .await
    {
        tracing::warn!("Not able to send message: {:?}", why);
    }
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
