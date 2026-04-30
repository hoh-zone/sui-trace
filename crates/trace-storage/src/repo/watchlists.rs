use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::Row;
use trace_common::{Error, error::Result};
use uuid::Uuid;

use crate::Db;

#[derive(Debug, Clone)]
pub struct Watchlist {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub target_type: String, // address | package | protocol
    pub target_id: String,
    pub rules: Value,
    pub channels: Value,
    pub created_at: DateTime<Utc>,
}

pub struct WatchlistRepo<'a> {
    db: &'a Db,
}

impl<'a> WatchlistRepo<'a> {
    pub fn new(db: &'a Db) -> Self {
        Self { db }
    }

    pub async fn create(&self, w: &Watchlist) -> Result<()> {
        sqlx::query(
            r#"INSERT INTO watchlists (id, user_id, name, target_type, target_id, rules, channels, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
        )
        .bind(w.id)
        .bind(w.user_id)
        .bind(&w.name)
        .bind(&w.target_type)
        .bind(&w.target_id)
        .bind(&w.rules)
        .bind(&w.channels)
        .bind(w.created_at)
        .execute(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn list_for_user(&self, user_id: Uuid) -> Result<Vec<Watchlist>> {
        let rows = sqlx::query(
            r#"SELECT id, user_id, name, target_type, target_id, rules, channels, created_at
               FROM watchlists WHERE user_id = $1 ORDER BY created_at DESC"#,
        )
        .bind(user_id)
        .fetch_all(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(rows.into_iter().map(row_to_wl).collect())
    }

    pub async fn list_by_target(
        &self,
        target_type: &str,
        target_id: &str,
    ) -> Result<Vec<Watchlist>> {
        let rows = sqlx::query(
            r#"SELECT id, user_id, name, target_type, target_id, rules, channels, created_at
               FROM watchlists WHERE target_type = $1 AND target_id = $2"#,
        )
        .bind(target_type)
        .bind(target_id)
        .fetch_all(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(rows.into_iter().map(row_to_wl).collect())
    }

    pub async fn delete(&self, id: Uuid, user_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM watchlists WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .execute(self.db.pool())
            .await
            .map_err(|e| Error::Database(e.to_string()))?;
        Ok(())
    }
}

fn row_to_wl(r: sqlx::postgres::PgRow) -> Watchlist {
    Watchlist {
        id: r.get(0),
        user_id: r.get(1),
        name: r.get(2),
        target_type: r.get(3),
        target_id: r.get(4),
        rules: r.get(5),
        channels: r.get(6),
        created_at: r.get(7),
    }
}
