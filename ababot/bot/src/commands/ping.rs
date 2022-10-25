use serenity::{model::prelude::interaction::application_command::CommandDataOption, builder::CreateApplicationCommand};



pub fn run(_options: &[CommandDataOption]) -> String {
    "Pong!".to_string()
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("ping").description("A ping command")
}