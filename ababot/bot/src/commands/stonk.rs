use crate::types::stonk::Stonk;
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        command::CommandOptionType,
        interaction::{
            application_command::ApplicationCommandInteraction, InteractionResponseType,
        },
    },
    prelude::Context,
};
use yahoo_finance_api as yahoo;

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) {
    let mut stonk: String = String::new();
    for opt in &command.data.options {
        if opt.name == "ticker" {
            let ticker = opt
                .value
                .as_ref()
                .map(|v| v.as_str())
                .flatten()
                .unwrap_or("AAPL");
            let stonk_history = get_last_stonk(ticker).await;
            let first: String = match stonk_history {
                Ok(stonk) => stonk.close.to_string(),
                Err(e) => {
                    println!("Error: {}", e);
                    "No stonks found".to_string()
                }
            };
            stonk = format!("{}: {}", opt.value.as_ref().unwrap(), first);
        }
    }
    if let Err(why) = command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.content(stonk))
        })
        .await
    {
        tracing::warn!("Failed to run command: {}", why);
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("stonk")
        .description("When you need stonk")
        .create_option(|option| {
            option
                .name("ticker")
                .description("The symbol of the stonk you want to get")
                .kind(CommandOptionType::String)
                .required(true)
        })
}

pub async fn get_latest_stonks(stonk_name: &str) -> Result<Vec<Stonk>, Box<dyn std::error::Error>> {
    let provider = yahoo::YahooConnector::new();
    let resp = provider
        .get_latest_quotes(stonk_name, "1m")
        .await?
        .quotes()?
        .iter()
        .map(|quote| Stonk::from(quote))
        .collect();
    Ok(resp)
}
pub async fn get_last_stonk(stonk_name: &str) -> Result<Stonk, Box<dyn std::error::Error>> {
    let provider = yahoo::YahooConnector::new();
    let resp = provider.get_latest_quotes(stonk_name, "1m").await?;
    let quote = resp.last_quote()?;
    let stonk = Stonk::from(&quote);
    Ok(stonk)
}
