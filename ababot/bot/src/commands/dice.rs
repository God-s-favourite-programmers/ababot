use std::mem::swap;

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

#[cfg(feature = "dice")]
use rand::{thread_rng, Rng};
use tracing::instrument;

#[instrument(skip(ctx, command))]
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
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
            }
            if option.name == "max" {
                max = option
                    .value
                    .as_ref()
                    .and_then(|v| v.as_i64())
                    .unwrap_or(100);
            }
        }
        if max < min {
            swap(&mut max, &mut min);
        }
        let dice_roll = if min == max {
            format!("{} to {} is not a valid range", min, max)
        } else {
            let mut rng = thread_rng();
            rng.gen_range(min..max).to_string()
        };
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
        tracing::debug!("Registering command dice");
        command
            .name("dice")
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
    #[cfg(not(feature = "dice"))]
    {
        command
    }
}
