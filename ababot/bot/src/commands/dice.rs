use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        command::CommandOptionType, interaction::application_command::CommandDataOption,
    },
};

#[cfg(feature = "dice")]
use rand::{thread_rng, Rng};

pub async fn run(options: &[CommandDataOption]) -> String {
    #[cfg(feature = "dice")]
    {
        let mut min = 0;
        let mut max = 100;

        for option in options {
            if option.name == "min" {
                min = option
                    .value
                    .as_ref()
                    .map(|v| v.as_i64())
                    .flatten()
                    .unwrap_or(0);
            }
            if option.name == "max" {
                max = option
                    .value
                    .as_ref()
                    .map(|v| v.as_i64())
                    .flatten()
                    .unwrap_or(100);
            }
        }
        if min == max {
            return format!("{} to {} is not a valid range", min, max);
        } else if max < min {
            let tmp = max;
            max = min;
            min = tmp;
        }
        let mut rng = thread_rng();
        rng.gen_range(min..max).to_string()
    }
    #[cfg(not(feature = "dice"))]
    {
        "Command disabled".to_string()
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    #[cfg(feature = "dice")]
    {
        let c = command
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
            });
        tracing::debug!("Registered Dice command");
        c
    }
    #[cfg(not(feature = "dice"))]
    {
        command
    }
}
