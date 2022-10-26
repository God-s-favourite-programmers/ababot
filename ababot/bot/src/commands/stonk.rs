use crate::types::stonk::Stonk;
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{interaction::application_command::CommandDataOption, command::CommandOptionType},
};
use yahoo_finance_api as yahoo;


pub async fn run(options: &[CommandDataOption]) -> String {
    let mut stonk: String = String::new();
    for opt in options {
        if opt.name == "ticker" {
            let ticker = opt.value.as_ref().map(|v| v.as_str()).flatten().unwrap_or("AAPL");
            println!("Ticker: {}", ticker);
            let stonk_history = get_last_stonk(ticker).await;
            println!("{:?}", stonk_history);
            let first:String = match stonk_history {
                Ok(stonk) => stonk.close.to_string(),
                Err(e) => {
                    println!("Error: {}", e);
                    return "No stonks found".to_string()
                },
            };
            stonk = format!("{}: {}", opt.value.as_ref().unwrap(), first);
        }
    }
    stonk
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