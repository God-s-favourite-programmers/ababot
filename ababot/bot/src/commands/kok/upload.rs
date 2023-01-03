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

pub async fn save_big(ctx: &Context, command: &ModalSubmitInteraction) {
    let string_answers = command.data.components[0]
        .components
        .iter()
        .map(|comp| match comp {
            ActionRowComponent::InputText(input) => input.value.clone(),
            _ => panic!("Not an input text"),
        })
        .collect::<Vec<String>>();
    let name = match string_answers.get(0) {
        Some(name) => name,
        None => return,
    };
    let url = match string_answers.get(1) {
        Some(url) => url,
        None => return,
    };
    // File is big. Save the url
    let page = reqwest::get(url).await.unwrap().text().await.unwrap();
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
        .create_interaction_response(&ctx.http, |m| {
            m.kind(InteractionResponseType::DeferredChannelMessageWithSource)
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

pub async fn create_modal(ctx: &Context, command: &ApplicationCommandInteraction) {
    match command
        .create_interaction_response(&ctx.http, |m| {
            m.kind(InteractionResponseType::Modal)
                .interaction_response_data(|d| {
                    d.content("Please select the file you want to download")
                        .custom_id("download")
                        .title("Download")
                        .components(|c| {
                            c.create_action_row(|row| {
                                row.create_input_text(|menu| {
                                    menu.custom_id("kok")
                                        .placeholder("Download link")
                                        .label("Download link")
                                        .style(InputTextStyle::Short)
                                })
                            })
                            .create_action_row(|row| {
                                row.create_input_text(|menu| {
                                    menu.custom_id("name")
                                        .placeholder("Name")
                                        .label("Name")
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
            println!("Error fetching: {:?}", e);
            tracing::warn!("Not able to create modal")
        }
    };
}
