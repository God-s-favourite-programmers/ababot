use std::sync::Arc;

use scraper::{Html, Selector};
use serenity::{
    model::prelude::interaction::message_component::MessageComponentInteraction, prelude::Context,
};

struct Ingredient {
    name: String,
    amount: String,
    unit: String,
}

pub async fn create_recipe_post(
    ctx: &Context,
    command: Arc<MessageComponentInteraction>,
    url: String,
) {
    let client = reqwest::Client::new();
    let response = if let Ok(response) = client.get(&url).send().await {
        response
    } else {
        tracing::error!("Error fetching recipe");
        error(ctx, command, "Error fetching recipe".to_string()).await;
        return;
    };
    let body = if let Ok(body) = response.text().await {
        body
    } else {
        tracing::error!("Error parsing recipe");
        error(ctx, command, "Error parsing recipe".to_string()).await;
        return;
    };
    let ingredients = match get_ingredients(&body) {
        Ok(ingredients) => ingredients,
        Err(e) => {
            error(ctx, command, e).await;
            return;
        }
    };
    let steps = match get_steps(&body) {
        Ok(steps) => steps,
        Err(e) => {
            error(ctx, command, e).await;
            return;
        }
    };
    let name = match get_name(&body) {
        Ok(name) => name,
        Err(e) => {
            error(ctx, command, e).await;
            return;
        }
    };
    if let Err(why) = command
        .create_followup_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title(name)
                    .field("Fremgangsmåte", steps, false)
                    .field("Ingredienser", "­", false) // Invisible character in value field to make discord happy
                    .fields(ingredients.iter().map(|i| {
                        (
                            i.name.clone(),
                            format!("{} {}", i.amount.clone(), i.unit.clone()),
                            true,
                        )
                    }))
            })
        })
        .await
    {
        tracing::warn!("Error sending recipe: {:?}", why);
    }
}

fn get_name(body: &String) -> Result<String, String> {
    let document = Html::parse_document(body);
    let selector = Selector::parse("h1[itemprop='name']").map_err(|e| e.to_string())?;
    let elements = document
        .select(&selector)
        .next()
        .ok_or("No elements found")?;
    Ok(elements.text().collect::<Vec<_>>().join(" "))
}

fn get_steps(body: &String) -> Result<String, String> {
    let document = Html::parse_document(body);
    let selector =
        Selector::parse("span[itemprop='recipeInstructions']").map_err(|e| e.to_string())?;
    let elements = document
        .select(&selector)
        .next()
        .ok_or("No elements found")?;
    Ok(elements.inner_html().replace("<br>", ""))
}

fn get_ingredients(body: &String) -> Result<Vec<Ingredient>, String> {
    let document = Html::parse_document(body);
    let selector = Selector::parse("table.table_ingredienser_bilde").map_err(|e| e.to_string())?;
    let tr_selector = Selector::parse("tr").map_err(|e| e.to_string())?;

    let elements = document
        .select(&selector)
        .next()
        .ok_or("No elements found")?;

    let mut ingredients = Vec::new();
    for child in elements.select(&tr_selector) {
        let inner_selector = Selector::parse("td").map_err(|e| e.to_string())?;
        let mut iterator = child.select(&inner_selector);
        let mut temp = Vec::new();
        while let Some(ingredient) = iterator.next() {
            temp.push(ingredient.text().collect::<Vec<_>>().join(" "));
        }
        if temp.len() == 3
            && !temp.get(0).unwrap().is_empty()
            && !temp.get(1).unwrap().is_empty()
            && !temp.get(2).unwrap().is_empty()
        {
            ingredients.push(Ingredient {
                amount: temp.get(0).unwrap().to_string(),
                unit: temp.get(1).unwrap().to_string(),
                name: temp.get(2).unwrap().to_string(),
            });
        } else {
            continue;
        }
    }
    Ok(ingredients)
}

async fn error(ctx: &Context, command: Arc<MessageComponentInteraction>, error: String) {
    if let Err(why) = command
        .create_followup_message(&ctx.http, |m| m.content(format!("Error: {}", error)))
        .await
    {
        tracing::warn!("Error sending error message: {:?}", why);
    }
}
