use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::Row;
use trace_common::{Error, error::Result};
use uuid::Uuid;

use crate::Db;

pub struct AlertRepo<'a> {
    db: &'a Db,
}

impl<'a> AlertRepo<'a> {
    pub fn new(db: &'a Db) -> Self {
        Self { db }
    }

    pub async fn record(
        &self,
        id: Uuid,
        user_id: Option<Uuid>,
        watchlist_id: Option<Uuid>,
        rule_id: &str,
        payload: &Value,
    ) -> Result<()> {
        sqlx::query(
            r#"INSERT INTO alert_events (id, user_id, watchlist_id, rule_id, fired_at, payload, delivered, attempts)
               VALUES ($1, $2, $3, $4, NOW(), $5, false, 0)"#,
        )
        .bind(id)
        .bind(user_id)
        .bind(watchlist_id)
        .bind(rule_id)
        .bind(payload)
        .execute(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn mark_delivered(&self, id: Uuid) -> Result<()> {
        sqlx::query(
            "UPDATE alert_events SET delivered = true, attempts = attempts + 1 WHERE id = $1",
        )
        .bind(id)
        .execute(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn mark_failed(&self, id: Uuid, reason: &str) -> Result<()> {
        sqlx::query(
            "UPDATE alert_events SET attempts = attempts + 1, last_error = $2 WHERE id = $1",
        )
        .bind(id)
        .bind(reason)
        .execute(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(())
    }

    /// Latest alert events across all watchlists. Useful for the public live
    /// feed surfaced on the explorer home page.
    pub async fn recent_feed(&self, limit: i64) -> Result<Vec<AlertRow>> {
        let rows = sqlx::query(
            r#"SELECT id, user_id, watchlist_id, rule_id, fired_at, payload, delivered
               FROM alert_events ORDER BY fired_at DESC LIMIT $1"#,
        )
        .bind(limit)
        .fetch_all(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(rows
            .into_iter()
            .map(|r| AlertRow {
                id: r.get(0),
                user_id: r.get(1),
                watchlist_id: r.get(2),
                rule_id: r.get(3),
                fired_at: r.get(4),
                payload: r.get(5),
                delivered: r.get(6),
            })
            .collect())
    }

    pub async fn recent_for_user(&self, user_id: Uuid, limit: i64) -> Result<Vec<AlertRow>> {
        let rows = sqlx::query(
            r#"SELECT id, user_id, watchlist_id, rule_id, fired_at, payload, delivered
               FROM alert_events WHERE user_id = $1 ORDER BY fired_at DESC LIMIT $2"#,
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(rows
            .into_iter()
            .map(|r| AlertRow {
                id: r.get(0),
                user_id: r.get(1),
                watchlist_id: r.get(2),
                rule_id: r.get(3),
                fired_at: r.get(4),
                payload: r.get(5),
                delivered: r.get(6),
            })
            .collect())
    }
}

#[derive(Debug, Clone)]
pub struct AlertRow {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub watchlist_id: Option<Uuid>,
    pub rule_id: String,
    pub fired_at: DateTime<Utc>,
    pub payload: Value,
    pub delivered: bool,
}
