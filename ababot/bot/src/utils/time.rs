use std::time::Duration;

use chrono::{Datelike, Weekday, Days};
use chrono_tz::{Europe::Oslo, Tz};
use tracing::instrument;

#[derive(Debug, Clone, Copy)]
pub enum Interval {
    EveryDelta(std::time::Duration),
    EveryDeltaStartAt(std::time::Duration, chrono::DateTime<Tz>),
}

pub const WEEK_AS_SECONDS: u64 = 604800;
pub const DAY_AS_SECONDS: u64 = 86400;

#[derive(Debug, Clone, Copy)]
pub struct Time {
    hour: u8,
    minute: u8,
    second: u8,
}

impl Time {
    pub fn new(hour: u8, minute: u8, second: u8) -> Option<Self> {
        if hour > 23 {
            return None;
        }

        if minute > 59 {
            return None;
        }

        if second > 59 {
            return None;
        }

        Some(Time {
            hour,
            minute,
            second,
        })
    }

    pub fn new_unchecked(hour: u8, minute: u8, second: u8) -> Self {
        Time::new(hour, minute, second).unwrap_or_else(|| {
            tracing::error!("Illegal time");
            panic!("Illegal time");
        })
    }

    pub fn today(&self) -> Option<chrono::NaiveDateTime> {
        chrono::Utc::now().date_naive().and_hms_opt(
            self.hour as u32,
            self.minute as u32,
            self.second as u32,
        )
    }

    pub fn nearest(&self) -> Option<chrono::DateTime<Tz>> {
        let today = self.today()?.and_local_timezone(Oslo).earliest()?;
        let now = chrono::Utc::now().with_timezone(&Oslo);
        if now > today {
            today
                .date_naive()
                .succ_opt()?
                .and_hms_opt(self.hour as u32, self.minute as u32, self.second as u32)?
                .and_local_timezone(Oslo)
                .earliest()
        } else {
            Some(today)
        }
    }

    pub fn nearest_unchecked(&self) -> chrono::DateTime<Tz> {
        Time::new(self.hour, self.minute, self.second)
            .unwrap_or_else(|| {
                tracing::error!("Illegal time");
                panic!("Illegal time")
            })
            .nearest()
            .unwrap_or_else(|| {
                tracing::error!("Time is unrepresentable");
                panic!("Time is unrepresentable");
            })
    }
}

impl Default for Time {
    fn default() -> Self {
        Time {
            hour: 7,
            minute: 0,
            second: 0,
        }
    }
}

pub struct TimeBuilder {
    hour: Option<u8>,
    minute: Option<u8>,
    second: Option<u8>,
}

impl TimeBuilder {
    pub fn new() -> Self {
        TimeBuilder {
            hour: None,
            minute: None,
            second: None,
        }
    }

    pub fn hour(mut self, hour: u8) -> Self {
        self.hour = Some(hour);
        self
    }

    pub fn minute(mut self, minute: u8) -> Self {
        self.minute = Some(minute);
        self
    }

    pub fn second(mut self, second: u8) -> Self {
        self.second = Some(second);
        self
    }

    pub fn build(self) -> Option<Time> {
        self.into()
    }
}

impl Default for TimeBuilder {
    fn default() -> Self {
        TimeBuilder::new()
    }
}

impl From<TimeBuilder> for Option<Time> {
    fn from(builder: TimeBuilder) -> Self {
        builder.into()
    }
}

/// Schedules an action to be run daily at the provided time
/// Utilizes [schedule] so the same restrictions apply
/// # Panics
/// This function panics if the provided time is illegal
/// i.e. hour > 23 or minute / second > 59
#[instrument(skip(action))]
pub async fn daily<Action, Async>(time: Time, action: Action)
where
    Action: Fn() -> Async,
    Async: std::future::Future<Output = ()>,
{
    schedule(
        Interval::EveryDeltaStartAt(Duration::from_secs(DAY_AS_SECONDS), time.nearest_unchecked()),
        action,
    )
    .await;
}

/// Schedules an action to be run weekly at the provided time
/// Utilizes [schedule] so the same restrictions apply
/// # Panics
/// This function panics if the provided time is illegal
/// i.e. hour > 23 or minute / second > 59
#[instrument(skip(action))]
pub async fn weekly<Action, Async>(time: Time, day: Weekday, action: Action)
where
    Action: Fn() -> Async,
    Async: std::future::Future<Output = ()>,
{
    let mut nearest_time = time.nearest_unchecked();
    while nearest_time.weekday() != day {
        nearest_time = nearest_time.checked_add_days(Days::new(1)).unwrap_or_else(|| {
            tracing::error!("Date cannot be represented");
            panic!("Date cannot be represented");
        })
    }
    
    schedule(
        Interval::EveryDeltaStartAt(Duration::from_secs(WEEK_AS_SECONDS), nearest_time),
        action,
    )
    .await;
}

/// Schedule an action to be repeated
/// This function will never return, as it is stuck in an infinite loop
/// Only way it exits is through a panic
/// # Panics
/// This function will panic when providing [Interval::EveryDeltaStartAt] iwth a start time that is behind current time
#[instrument(skip(action))]
pub async fn schedule<Action, Async>(time: Interval, action: Action)
where
    Action: Fn() -> Async,
    Async: std::future::Future<Output = ()>,
{
    let now = chrono::Utc::now().with_timezone(&Oslo);
    match time {
        Interval::EveryDelta(delta) => {
            let mut interval_timer = tokio::time::interval(delta);
            loop {
                interval_timer.tick().await;
                action().await;
            }
        }
        Interval::EveryDeltaStartAt(delta, time) => {
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
