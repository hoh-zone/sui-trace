use chrono::{DateTime, Utc};
use sqlx::Row;
use trace_common::{Error, error::Result, model::Event};

use crate::Db;

pub struct EventRepo<'a> {
    db: &'a Db,
}

impl<'a> EventRepo<'a> {
    pub fn new(db: &'a Db) -> Self {
        Self { db }
    }

    pub async fn insert_many(&self, events: &[Event]) -> Result<u64> {
        if events.is_empty() {
            return Ok(0);
        }
        let mut tx = self
            .db
            .pool()
            .begin()
            .await
            .map_err(|e| Error::Database(e.to_string()))?;
        let mut count = 0u64;
        for e in events {
            sqlx::query(
                r#"
                INSERT INTO events
                    (tx_digest, event_seq, package_id, module, event_type, sender, timestamp, payload)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT (tx_digest, event_seq) DO NOTHING
                "#,
            )
            .bind(&e.tx_digest)
            .bind(e.event_seq as i32)
            .bind(&e.package_id)
            .bind(&e.module)
            .bind(&e.event_type)
            .bind(&e.sender)
            .bind(e.timestamp)
            .bind(&e.payload)
            .execute(&mut *tx)
            .await
            .map_err(|err| Error::Database(err.to_string()))?;
            count += 1;
        }
        tx.commit()
            .await
            .map_err(|e| Error::Database(e.to_string()))?;
        Ok(count)
    }

    pub async fn list_by_tx(&self, digest: &str) -> Result<Vec<Event>> {
        let rows = sqlx::query(
            r#"SELECT tx_digest, event_seq, package_id, module, event_type, sender, timestamp, payload
               FROM events WHERE tx_digest = $1 ORDER BY event_seq"#,
        )
        .bind(digest)
        .fetch_all(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(rows.into_iter().map(row_to_event).collect())
    }

    pub async fn list_by_package(&self, package: &str, limit: i64) -> Result<Vec<Event>> {
        let rows = sqlx::query(
            r#"SELECT tx_digest, event_seq, package_id, module, event_type, sender, timestamp, payload
               FROM events WHERE package_id = $1 ORDER BY timestamp DESC LIMIT $2"#,
        )
        .bind(package)
        .bind(limit)
        .fetch_all(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(rows.into_iter().map(row_to_event).collect())
    }

    pub async fn list_by_address(&self, address: &str, limit: i64) -> Result<Vec<Event>> {
        let rows = sqlx::query(
            r#"SELECT tx_digest, event_seq, package_id, module, event_type, sender, timestamp, payload
               FROM events WHERE sender = $1 ORDER BY timestamp DESC LIMIT $2"#,
        )
        .bind(address)
        .bind(limit)
        .fetch_all(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(rows.into_iter().map(row_to_event).collect())
    }
}

fn row_to_event(r: sqlx::postgres::PgRow) -> Event {
    Event {
        tx_digest: r.get(0),
        event_seq: r.get::<i32, _>(1) as u32,
        package_id: r.get(2),
        module: r.get(3),
        event_type: r.get(4),
        sender: r.get(5),
        timestamp: r.get::<DateTime<Utc>, _>(6),
        payload: r.get(7),
    }
}
