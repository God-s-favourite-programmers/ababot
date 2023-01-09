use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        component::InputTextStyle,
        interaction::{
            application_command::ApplicationCommandInteraction, InteractionResponseType,
        },
    },
    prelude::Context,
};

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) {
    if let Err(why) = command
        .create_interaction_response(&ctx.http, |m| {
            m.kind(InteractionResponseType::Modal)
                .interaction_response_data(|d| {
                    d.content("Food")
                        .custom_id("food")
                        .title("Enter ingredients. På norsk :D")
                        .components(|c| {
                            c.create_action_row(|row| {
                                row.create_input_text(|menu| {
                                    menu.custom_id("ing1")
                                        .placeholder("Føøøød")
                                        .label("Ingrediens 1")
                                        .style(InputTextStyle::Short)
                                        .required(false)
                                })
                            })
                            .create_action_row(|row| {
                                row.create_input_text(|menu| {
                                    menu.custom_id("ing2")
                                        .placeholder("Føøøød")
                                        .label("Ingrediens 2")
                                        .style(InputTextStyle::Short)
                                        .required(false)
                                })
                            })
                            .create_action_row(|row| {
                                row.create_input_text(|menu| {
                                    menu.custom_id("ing3")
                                        .placeholder("Føøøød")
                                        .label("Ingrediens 3")
                                        .style(InputTextStyle::Short)
                                        .required(false)
                                })
                            })
                            .create_action_row(|row| {
                                row.create_input_text(|menu| {
                                    menu.custom_id("ing4")
                                        .placeholder("Føøøød")
                                        .label("Ingrediens 4")
                                        .style(InputTextStyle::Short)
                                        .required(false)
                                })
                            })
                            .create_action_row(|row| {
                                row.create_input_text(|menu| {
                                    menu.custom_id("ing5")
                                        .placeholder("Føøøød")
                                        .label("Ingrediens 5")
                                        .style(InputTextStyle::Short)
                                        .required(false)
                                })
                            })
                        })
                })
        })
        .await
    {
        tracing::warn!("Error sending modal: {}", why);
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    tracing::debug!("Registering command knock");
    command
        .name("food")
        .description("Find a recipe based on your ingredients")
}
