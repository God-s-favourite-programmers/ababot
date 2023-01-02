use serenity::{
    model::prelude::interaction::application_command::ApplicationCommandInteraction,
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
    let download_url = local_parse(page);
    let download_file = reqwest::get(&download_url).await.unwrap().bytes().await.unwrap();
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

async fn get_donwload_link(ctx: &Context, command: &ApplicationCommandInteraction) {
    command.channel_id.send_message(&ctx.http, |m| {
        m// A lot todo
    }).await.unwrap();
}
