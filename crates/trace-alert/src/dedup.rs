//! Deduplication helper backed by the `alert_dedup` table.
//!
//! Returns `true` if the key was newly inserted (caller should fire alert),
//! `false` if it already existed within the dedupe window.

use chrono::{Duration, Utc};
use sqlx::Row;
use trace_common::{Error, error::Result};
use trace_storage::Db;

pub async fn try_acquire(db: &Db, key: &str, window_secs: i64) -> Result<bool> {
    let cutoff = Utc::now() - Duration::seconds(window_secs);
    sqlx::query("DELETE FROM alert_dedup WHERE fired_at < $1")
        .bind(cutoff)
        .execute(db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

    let row = sqlx::query(
        r#"INSERT INTO alert_dedup (key) VALUES ($1)
           ON CONFLICT (key) DO NOTHING
           RETURNING 1"#,
    )
    .bind(key)
    .fetch_optional(db.pool())
    .await
    .map_err(|e| Error::Database(e.to_string()))?;
    let _ = row.as_ref().map(|r| r.try_get::<i32, _>(0));
    Ok(row.is_some())
}
