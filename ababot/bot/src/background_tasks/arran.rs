use serenity::{
    model::prelude::{Member, UserId},
    prelude::Context,
};

use rand::Rng;
use std::{sync::Arc, time::Duration};
use tracing::instrument;

use crate::utils::{
    get_channel_id,
    time::{schedule, Interval},
};

#[instrument(skip(ctx))]
pub async fn run(ctx: Arc<Context>) {
    schedule(
        Interval::EveryDelta(Duration::from_secs(60 * 60)),
        || async { find_out(ctx.clone()).await },
    )
    .await;
}

#[instrument(skip(ctx))]
async fn get_him(ctx: Arc<Context>) -> Result<Option<Member>, ()> {
    let mut a = None;
    for user in get_channel_id("general", &ctx.http)
        .await
        .map_err(|_e| {
            tracing::debug!("Failed to get channel id");
        })?
        .to_channel((&ctx.cache, ctx.http.as_ref()))
        .await
        .map_err(|_e| {
            tracing::debug!("Failed to get channel");
        })?
        .guild()
        .ok_or_else(|| {
            tracing::debug!("Failed to get guild");
        })?
        .members(&ctx.cache)
        .await
        .map_err(|_e| {
            tracing::debug!("Failed to get users");
        })?
    {
        if user.user.id == UserId(114413921955348482) {
            a = Some(user);
        }
    }
    Ok(a)
}

#[instrument(skip(ctx))]
async fn find_out(ctx: Arc<Context>) {
    let _r = inner(ctx).await;
}

#[instrument(skip(ctx))]
async fn inner(ctx: Arc<Context>) -> Result<(), ()> {
    let a = get_him(ctx.clone())
        .await?
        .ok_or_else(|| tracing::debug!("Failed to get him"))?;
    if rand::thread_rng().gen_bool(1.0 / 10.0) {
        // Nice nickname you got there
        tracing::info!("Changing nickname");
        let mut nick: String = String::new();

        a.nick
            .as_ref()
            .ok_or_else(|| tracing::debug!("He has no name"))?
            .to_lowercase()
            .chars()
            .map(|c| {
                if rand::thread_rng().gen_bool(1.0 / 3.0) {
                    c.to_uppercase()
                        .next()
                        .ok_or_else(|| tracing::error!("Failed to capitalize {}", c))
                        .unwrap()
                } else {
                    c
                }
            })
            .for_each(|c| nick.push(c));
        let _r = a.edit(&ctx.http, |u| u.nickname(nick)).await;
        return Ok(());
    }

    if rand::thread_rng().gen_bool(1.0 / 10.0) {
        // Missclick
        tracing::info!("Disconnecting");
        let _r = a.disconnect_from_voice(&ctx.http).await;
        return Ok(());
    }

    if rand::thread_rng().gen_bool(1.0 / 10.0) {
        // Missclick
        tracing::info!("Moving to afk");
        let _r = a
            .move_to_voice_channel(
                &ctx.http,
                get_channel_id("afk", &ctx.http)
                    .await
                    .map_err(|_e| tracing::warn!("Failed to get afk channel"))?,
            )
            .await;
        return Ok(());
    }

    Ok(())
}
