use rand::Rng;
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        command::CommandOptionType, interaction::application_command::CommandDataOption,
    },
};

#[cfg(feature = "dice")]
use rand;

pub fn run(options: &[CommandDataOption]) -> String {
    #[cfg(feature = "dice")]
    {
        let min;
        let max;

        for option in options {
            if option.name == "min" {
                min = option.value.map(|v| v.as_i64()).flatten().unwrap_or(0);
            }
            if option.name == "max" {
                max = option.value.map(|v| v.as_i64()).flatten().unwrap_or(100);
            }
        }
        let mut rng = rand::thread_rng();
        rng.gen_range(min..max).to_string()
    }
    #[cfg(not(feature = "dice"))]
    {
        "Command is turned off".to_string()
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    #[cfg(feature = "dice")]
    {
        command
            .name("Dice")
            .description("Get a random number")
            .create_option(|option| {
                option
                    .name("min")
                    .description("The minimum value for the number generator")
                    .kind(CommandOptionType::Integer)
                    .min_int_value(0)
                    .max_int_value(99)
                    .required(false)
            })
            .create_option(|option| {
                option
                    .name("max")
                    .description("The maximum value for the number generator")
                    .kind(CommandOptionType::Integer)
                    .min_int_value(1)
                    .max_int_value(100)
                    .required(false)
            })
    }
}
