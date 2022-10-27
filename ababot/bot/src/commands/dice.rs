use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        command::{self, CommandOptionType},
        interaction::application_command,
        interaction::{
            application_command::{ApplicationCommandInteraction, CommandDataOption},
            InteractionResponseType,
        },
    },
    prelude::Context,
};

#[cfg(feature = "dice")]
use rand::{thread_rng, Rng};

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) {
    #[cfg(feature = "dice")]
    {
        let mut min = 0;
        let mut max = 100;

        for option in &command.data.options {
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
        let dice_roll;
        if max < min {
            let tmp = max;
            max = min;
            min = tmp;
        }
        if min == max {
            dice_roll = format!("{} to {} is not a valid range", min, max);
        } else {
            let mut rng = thread_rng();
            dice_roll = rng.gen_range(min..max).to_string();
        }
        if let Err(why) = command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| message.content(dice_roll))
            })
            .await
        {
            tracing::warn!("Failed to run command: {}", why);
        }
    }
    // Feature noe deprecated, fix before merge
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
