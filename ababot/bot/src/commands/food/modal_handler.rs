use chrono::Duration;
use rand::seq::SliceRandom;
use reqwest::Client;
use scraper::{Html, Selector};
use serenity::{
    model::prelude::{
        component::ActionRowComponent,
        interaction::{modal::ModalSubmitInteraction, InteractionResponseType},
    },
    prelude::Context,
};

use crate::commands::food::recipe_response::create_recipe_post;

const BASE_URL: &str = "https://www.matoppskrift.no/sider/sokemaskin.asp?valg=kjoleskap&type1=1";
#[derive(Clone)]
struct Food {
    name: String,
    url: String,
}

pub async fn handle_modal(ctx: &Context, command: &ModalSubmitInteraction) {
    command
        .create_interaction_response(&ctx.http, |m| {
            m.kind(InteractionResponseType::DeferredUpdateMessage)
        })
        .await
        .unwrap();

    let user_submitted_ingredients = &command
        .data
        .components
        .iter()
        .filter_map(|row| row.components.get(0))
        .filter_map(|component| match component {
            ActionRowComponent::InputText(input) => Some(input.value.clone()),
            _ => None,
        })
        .filter(|input| !input.is_empty())
        .collect::<Vec<String>>();

    let mut url = BASE_URL.to_string();
    for (i, ingredient) in user_submitted_ingredients.iter().enumerate() {
        match i {
            // Hurr durr not funny
            0 => url.push_str(&format!("&fritekst0={}", ingredient)),
            1 => url.push_str(&format!("&fritekst={}", ingredient)),
            _ => url.push_str(&format!("&fritekst_{}={}", i, ingredient)),
        }
    }

    let mut recipies = match get_recipes(&url).await {
        Ok(recipies) => recipies,
        Err(e) => {
            error(ctx, command, e).await;
            return;
        }
    };

    create_response(ctx, command, &mut recipies).await;
}

async fn create_response(
    ctx: &Context,
    command: &ModalSubmitInteraction,
    recipies: &mut Vec<Food>,
) {
    recipies.shuffle(&mut rand::thread_rng());
    let recipe_sublist = if let Some(recipie) = recipies.chunks(5).next() {
        recipie.to_vec()
    } else {
        error(ctx, command, "No recipies found".to_string()).await;
        return;
    };
    let channel_message =
        command
            .channel_id
            .send_message(&ctx.http, |m| {
                m.content(
                "Here are some recipies you can make with the ingredients you have in your fridge",
            )
            .components(|c| c.create_action_row(|row| {
                row.create_select_menu(|menu|{
                    menu.custom_id("food_recipie")
                    .placeholder("Select a recipie");
                    menu.options(|opt|{
                        recipe_sublist.iter().fold(opt, |opt, reci| {
                            opt.create_option(|o| {
                                o.label(&reci.name)
                                .value(&reci.url)
                                .description("Click to see the recipie")
                            })
                        })
                    })
                })
            }))
            })
            .await
            .unwrap();
    let listener = channel_message
        .await_component_interaction(ctx)
        .timeout(Duration::minutes(2).to_std().unwrap())
        .await;

    let (selected_recipe, interaction) = if let Some(interaction) = listener {
        interaction
            .create_interaction_response(&ctx.http, |m| {
                m.kind(InteractionResponseType::DeferredUpdateMessage)
            })
            .await
            .unwrap();
        (
            interaction.data.values.get(0).unwrap().as_str().to_string(),
            interaction,
        )
    } else {
        error(ctx, command, "No recipie selected".to_string()).await;
        return;
    };
    if let Err(why) = channel_message.delete(&ctx.http).await {
        tracing::warn!("Failed to delete message: {:?}", why);
    }
    create_recipe_post(ctx, interaction, selected_recipe).await;
}

async fn get_recipes(url: &str) -> Result<Vec<Food>, String> {
    let client = Client::new();
    let res = client.get(url).send().await.map_err(|e| e.to_string())?;
    let document = if res.status().is_success() {
        let body = res.text().await.map_err(|e| e.to_string())?;
        Html::parse_document(&body)
    } else {
        return Err(format!("Request failed with status code: {}", res.status()));
    };
    let primary_selector = Selector::parse("table.display").map_err(|e| e.to_string())?;
    let tr_selector = Selector::parse("a.resultat_sokemaskin").map_err(|e| e.to_string())?;
    let elements = document
        .select(&primary_selector)
        .next()
        .ok_or("Fant ingen oppskrift")?;

    let mut recipes = Vec::new();
    for element in elements.select(&tr_selector) {
        let name = element.value().attr("title").ok_or("No title found")?;
        let url = element.value().attr("href").ok_or("No href found")?;
        recipes.push(Food {
            name: name.to_string(),
            url: url.to_string(),
        });
    }
    Ok(recipes)
}

async fn error(ctx: &Context, command: &ModalSubmitInteraction, error: String) {
    if let Err(why) = command
        .create_followup_message(&ctx.http, |m| m.content(format!("Error: {}", error)))
        .await
    {
        tracing::warn!("Error sending error message: {:?}", why);
    }
}
