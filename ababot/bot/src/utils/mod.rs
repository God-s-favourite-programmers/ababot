pub mod gpgpu;

use chrono_tz::{Europe::Oslo, Tz};
use serenity::{
    http::Http,
    model::prelude::{ChannelId, GuildId},
};
use std::env;
use tracing::{instrument, metadata::LevelFilter, Level};
use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_subscriber::{
    fmt::format::{DefaultFields, FmtSpan, Format},
    FmtSubscriber,
};

pub fn get_logger() -> (
    FmtSubscriber<DefaultFields, Format, LevelFilter, NonBlocking>,
    WorkerGuard,
) {
    let appender = tracing_appender::rolling::daily("./var/log", "tibber-status-server");
    let (non_blocking_appender, guard) = tracing_appender::non_blocking(appender);

    let level = match env::var("LOG_LEVEL") {
        Ok(l) => match l.as_str() {
            "trace" => Level::TRACE,
            "debug" => Level::DEBUG,
            "info" => Level::INFO,
            "warn" => Level::WARN,
            "error" => Level::ERROR,
            _ => Level::INFO,
        },
        Err(_) => Level::INFO,
    };

    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_span_events(FmtSpan::NONE)
        .with_ansi(false)
        .with_max_level(level)
        .with_writer(non_blocking_appender)
        // completes the builder.
        .finish();

    (subscriber, guard)
}

#[derive(Debug, Clone, Copy)]
pub enum Time {
    EveryTime(chrono::DateTime<Tz>),
    EveryDelta(std::time::Duration),
    EveryDeltaStartAt(std::time::Duration, chrono::DateTime<Tz>),
}

pub const WEEK_AS_SECONDS: u64 = 604800;
pub const DAY_AS_SECONDS: u64 = 86400;

/// Schedule an action to be repeated
/// This function will never return, as it is stuck in an infinite loop
/// Only way it exits is through a panic
#[instrument(skip(action))]
pub async fn schedule<Action, Async>(time: Time, action: Action)
where
    Action: Fn() -> Async,
    Async: std::future::Future<Output = ()>,
{
    let now = chrono::Utc::now().with_timezone(&Oslo);
    match time {
        Time::EveryTime(mut time) => {
            loop {
                if now > time {
                    // We should start doing this tomorrow
                    time = time
                        .date_naive()
                        .succ_opt()
                        .map(|t| t.and_time(time.time()))
                        .unwrap_or_else(|| {
                            tracing::error!("Failed to get successor day of {:?}", time);
                            panic!("Failed to get successor day of {:?}", time)
                        })
                        .and_local_timezone(Oslo)
                        .earliest()
                        .unwrap_or_else(|| {
                            tracing::error!("Could not convert provided time to timezone");
                            panic!("Could not convert provided time to timezone");
                        })
                }

                let offset = time - now;
                match offset.to_std() {
                    Ok(o) => {
                        tokio::time::sleep(o).await;
                        action().await;
                    }
                    Err(_) => {
                        tracing::error!(
                            "Target time ({}) is behind current time ({})",
                            time,
                            chrono::offset::Local::now()
                        );
                        panic!("Target time is behind current time");
                    }
                }
            }
        }
        Time::EveryDelta(delta) => {
            let mut interval_timer = tokio::time::interval(delta);
            loop {
                interval_timer.tick().await;
                action().await;
            }
        }
        Time::EveryDeltaStartAt(delta, time) => {
            let offset = time - now;
            match offset.to_std() {
                Ok(o) => {
                    tokio::time::sleep(o).await;
                    let mut interval_timer = tokio::time::interval(delta);
                    loop {
                        interval_timer.tick().await;
                        action().await;
                    }
                }
                Err(_) => {
                    tracing::error!(
                        "Target time ({}) is behind current time ({})",
                        time,
                        chrono::offset::Local::now()
                    );
                    panic!("Target time is behind current time");
                }
            }
        }
    }
}

#[instrument(skip(http))]
pub async fn get_channel_id<T>(name: T, http: &Http) -> Result<ChannelId, &'static str>
where
    T: AsRef<str> + std::fmt::Debug,
{
    let guild = GuildId(
        env::var("GUILD_ID")
            .expect("Guild ID must be set as enviroment variable")
            .parse::<u64>()
            .expect("Guild ID must be a valid integer"),
    );
    let channels = guild.channels(http).await.map_err(|_e| {
        tracing::warn!("Failed to fetch channels for guild {}", guild);
        "Error fetching channels"
    })?;

    let first = channels
        .into_iter()
        .find(|(_, g)| g.name() == name.as_ref());
    match first {
        Some((c, _)) => Ok(c),
        None => {
            let r = guild.create_channel(http, |c| c.name(name.as_ref())).await;
            match r {
                Ok(c) => Ok(c.id),
                Err(_) => Err("Error creating guild"),
            }
        }
    }
}
