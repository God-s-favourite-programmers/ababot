use std::time::Duration;

use serenity::{
    model::prelude::{
        component::InputTextStyle, interaction::application_command::ApplicationCommandInteraction,
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
    // File is small. Save the pdf
    let mut file = File::create(format!("{}.pdf", name)).await.unwrap();
    file.write_all(&bytes).await.unwrap();
    command
        .create_followup_message(&ctx.http, |m| {
            m.content(format!("Saved file as {}.pdf", name))
        })
        .await
        .unwrap();
    // Save file
    file.sync_all().await.unwrap();
}

const TEMP_URL: &str = "https://anonfiles.com/j1GeG4P2y1/Julelapper_pdf";

pub async fn save_big(ctx: &Context, command: &ApplicationCommandInteraction, name: &str) {
    // File is big. Save the url
    let page = reqwest::get(TEMP_URL).await.unwrap().text().await.unwrap();
    let placeholder_string = get_download_link(ctx, command).await;
    println!("Placeholder string: {}", placeholder_string);
    let download_url = local_parse(page);
    let download_file = reqwest::get(&download_url)
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();
    let mut file = File::create(format!("{}.pdf", name)).await.unwrap();
    file.write_all(download_file.as_ref()).await.unwrap();
    command
        .create_followup_message(&ctx.http, |m| {
            m.content(format!("Saved file as {}.pdf", name))
        })
        .await
        .unwrap();
    file.sync_all().await.unwrap();
    println!("Saved file as {}.pdf", name);
}

fn local_parse(page: String) -> String {
    let document = scraper::Html::parse_document(&page);
    let selector = scraper::Selector::parse("#download-url").unwrap();
    document
        .select(&selector)
        .next()
        .unwrap()
        .value()
        .attr("href")
        .unwrap()
        .to_string()
}

async fn get_download_link(ctx: &Context, command: &ApplicationCommandInteraction) -> String {
    let channel_message = match command
        .channel_id
        .send_message(&ctx.http, |m| {
            m.content("Please send the download link")
                .components(|comp| {
                    comp.create_action_row(|row| {
                        row.create_input_text(|input| {
                            input
                                .custom_id("download_link")
                                .label("download")
                                .style(InputTextStyle::Short)
                                .placeholder("Download link")
                                .required(true)
                        })
                    })
                })
        })
        .await
    {
        Ok(m) => m,
        Err(e) => {
            println!("Error fetching: {:?}", e);
            return String::new();
        }
    };

    let mut download_link = String::new();
    let interaction = channel_message
        .await_component_interaction(&ctx)
        .timeout(Duration::from_secs(60))
        .await
        .unwrap();

    if let Some(data) = interaction.data.values.get(0) {
        download_link = data.as_str().to_string();
    } else {
        println!("No data");
    }

    download_link
}

