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
    EveryTime(chrono::DateTime<chrono::Local>),
    EveryDelta(std::time::Duration),
    EveryDeltaStartAt(std::time::Duration, chrono::DateTime<chrono::Local>),
}

/// Schedule an action to be repeated
/// This function will never return, as it is stuck in an infinite loop
/// Only way it exits is through a panic
#[instrument(skip(action))]
pub async fn schedule<Action, Async>(time: Time, action: Action)
where
    Action: Fn() -> Async,
    Async: std::future::Future<Output = ()>,
{
    match time {
        Time::EveryTime(mut time) => {
            loop {
                if chrono::offset::Local::now() > time {
                    // We should start doing this tomorrow
                    time = time.date().succ().and_time(time.time()).unwrap_or_else(|| {
                        tracing::error!("Failed to get successor day of {:?}", time);
                        panic!("Failed to get successor day of {:?}", time)
                    })
                }

                let offset = time - chrono::offset::Local::now();
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
            let offset = time - chrono::offset::Local::now();
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
