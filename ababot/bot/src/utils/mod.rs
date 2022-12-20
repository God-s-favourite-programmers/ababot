pub mod background_threads;
pub mod gpgpu;
pub mod time;

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
