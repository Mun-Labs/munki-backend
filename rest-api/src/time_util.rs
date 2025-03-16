use chrono::{DateTime, Timelike, Utc};

pub fn get_start_of_day(now: DateTime::<Utc>) -> DateTime::<Utc> {
    let start_of_day = now
        .with_hour(0)
        .and_then(|t| t.with_minute(0))
        .and_then(|t| t.with_second(0))
        .and_then(|t| t.with_nanosecond(0))
        .unwrap_or(now); // Fallback to now if adjustment fails
    start_of_day
}