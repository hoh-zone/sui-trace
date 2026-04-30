use chrono::{DateTime, Utc};
use sqlx::Row;
use trace_common::{Error, error::Result, model::Checkpoint};

use crate::Db;

pub struct CheckpointRepo<'a> {
    db: &'a Db,
}

impl<'a> CheckpointRepo<'a> {
    pub fn new(db: &'a Db) -> Self {
        Self { db }
    }

    pub async fn upsert(&self, cp: &Checkpoint) -> Result<()> {
        let ts: DateTime<Utc> = trace_common::time::from_millis(cp.timestamp_ms);
        sqlx::query(
            r#"
            INSERT INTO checkpoints (sequence_number, digest, timestamp, previous_digest, network_total_transactions, epoch)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (sequence_number) DO UPDATE SET
                digest = EXCLUDED.digest,
                timestamp = EXCLUDED.timestamp,
                network_total_transactions = EXCLUDED.network_total_transactions,
                epoch = EXCLUDED.epoch
            "#,
        )
        .bind(cp.sequence_number as i64)
        .bind(&cp.digest)
        .bind(ts)
        .bind(&cp.previous_digest)
        .bind(cp.network_total_transactions as i64)
        .bind(cp.epoch as i64)
        .execute(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn latest(&self) -> Result<Option<Checkpoint>> {
        let row = sqlx::query(
            r#"SELECT sequence_number, digest, timestamp, previous_digest, network_total_transactions, epoch
               FROM checkpoints ORDER BY sequence_number DESC LIMIT 1"#,
        )
        .fetch_optional(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        Ok(row.map(|r| Checkpoint {
            sequence_number: r.get::<i64, _>(0) as u64,
            digest: r.get(1),
            timestamp_ms: r.get::<DateTime<Utc>, _>(2).timestamp_millis(),
            previous_digest: r.get(3),
            network_total_transactions: r.get::<i64, _>(4) as u64,
            epoch: r.get::<i64, _>(5) as u64,
        }))
    }

    pub async fn get(&self, seq: u64) -> Result<Option<Checkpoint>> {
        let row = sqlx::query(
            r#"SELECT sequence_number, digest, timestamp, previous_digest, network_total_transactions, epoch
               FROM checkpoints WHERE sequence_number = $1"#,
        )
        .bind(seq as i64)
        .fetch_optional(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        Ok(row.map(|r| Checkpoint {
            sequence_number: r.get::<i64, _>(0) as u64,
            digest: r.get(1),
            timestamp_ms: r.get::<DateTime<Utc>, _>(2).timestamp_millis(),
            previous_digest: r.get(3),
            network_total_transactions: r.get::<i64, _>(4) as u64,
            epoch: r.get::<i64, _>(5) as u64,
        }))
    }

    pub async fn recent(&self, limit: i64) -> Result<Vec<Checkpoint>> {
        let rows = sqlx::query(
            r#"SELECT sequence_number, digest, timestamp, previous_digest, network_total_transactions, epoch
               FROM checkpoints ORDER BY sequence_number DESC LIMIT $1"#,
        )
        .bind(limit)
        .fetch_all(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| Checkpoint {
                sequence_number: r.get::<i64, _>(0) as u64,
                digest: r.get(1),
                timestamp_ms: r.get::<DateTime<Utc>, _>(2).timestamp_millis(),
                previous_digest: r.get(3),
                network_total_transactions: r.get::<i64, _>(4) as u64,
                epoch: r.get::<i64, _>(5) as u64,
            })
            .collect())
    }

    pub async fn watermark(&self, pipeline: &str) -> Result<Option<u64>> {
        let row = sqlx::query("SELECT high_watermark FROM pipeline_watermarks WHERE pipeline = $1")
            .bind(pipeline)
            .fetch_optional(self.db.pool())
            .await
            .map_err(|e| Error::Database(e.to_string()))?;
        Ok(row.map(|r| r.get::<i64, _>(0) as u64))
    }

    pub async fn set_watermark(&self, pipeline: &str, value: u64) -> Result<()> {
        sqlx::query(
            r#"INSERT INTO pipeline_watermarks (pipeline, high_watermark, updated_at)
               VALUES ($1, $2, NOW())
               ON CONFLICT (pipeline) DO UPDATE SET
                   high_watermark = EXCLUDED.high_watermark,
                   updated_at = NOW()"#,
        )
        .bind(pipeline)
        .bind(value as i64)
        .execute(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(())
    }
}
