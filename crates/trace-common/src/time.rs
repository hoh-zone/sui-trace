use chrono::{DateTime, TimeZone, Utc};

/// Convert a Sui millisecond timestamp into chrono UTC.
pub fn from_millis(ms: i64) -> DateTime<Utc> {
    Utc.timestamp_millis_opt(ms)
        .single()
        .unwrap_or_else(Utc::now)
}

pub fn now_millis() -> i64 {
    Utc::now().timestamp_millis()
}
