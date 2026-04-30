use chrono::{DateTime, Utc};
use sqlx::Row;
use trace_common::{Error, error::Result, model::TvlPoint};

use crate::Db;

pub struct TvlRepo<'a> {
    db: &'a Db,
}

impl<'a> TvlRepo<'a> {
    pub fn new(db: &'a Db) -> Self {
        Self { db }
    }

    pub async fn insert(&self, p: &TvlPoint) -> Result<()> {
        sqlx::query(
            r#"INSERT INTO tvl_snapshots (protocol_id, timestamp, tvl_usd, breakdown)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (protocol_id, timestamp) DO UPDATE SET
                   tvl_usd = EXCLUDED.tvl_usd,
                   breakdown = EXCLUDED.breakdown"#,
        )
        .bind(&p.protocol_id)
        .bind(p.timestamp)
        .bind(p.tvl_usd)
        .bind(&p.breakdown)
        .execute(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn history(
        &self,
        protocol_id: &str,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<TvlPoint>> {
        let rows = sqlx::query(
            r#"SELECT protocol_id, timestamp, tvl_usd, breakdown
               FROM tvl_snapshots
               WHERE protocol_id = $1 AND timestamp >= $2 AND timestamp <= $3
               ORDER BY timestamp"#,
        )
        .bind(protocol_id)
        .bind(from)
        .bind(to)
        .fetch_all(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(rows
            .into_iter()
            .map(|r| TvlPoint {
                protocol_id: r.get(0),
                timestamp: r.get(1),
                tvl_usd: r.get(2),
                breakdown: r.get(3),
            })
            .collect())
    }

    /// Single most recent TVL point for the given protocol, if any.
    pub async fn latest(&self, protocol_id: &str) -> Result<Option<TvlPoint>> {
        let row = sqlx::query(
            r#"SELECT protocol_id, timestamp, tvl_usd, breakdown
               FROM tvl_snapshots
               WHERE protocol_id = $1
               ORDER BY timestamp DESC LIMIT 1"#,
        )
        .bind(protocol_id)
        .fetch_optional(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(row.map(|r| TvlPoint {
            protocol_id: r.get(0),
            timestamp: r.get(1),
            tvl_usd: r.get(2),
            breakdown: r.get(3),
        }))
    }

    /// Compute the percentage drop between the latest and a previous point N seconds ago.
    pub async fn recent_drop_pct(
        &self,
        protocol_id: &str,
        window_secs: i64,
    ) -> Result<Option<f64>> {
        let row = sqlx::query(
            r#"
            WITH latest AS (
                SELECT tvl_usd, timestamp FROM tvl_snapshots
                WHERE protocol_id = $1 ORDER BY timestamp DESC LIMIT 1
            ), prev AS (
                SELECT tvl_usd FROM tvl_snapshots
                WHERE protocol_id = $1 AND timestamp <= (SELECT timestamp - ($2 || ' seconds')::interval FROM latest)
                ORDER BY timestamp DESC LIMIT 1
            )
            SELECT latest.tvl_usd, prev.tvl_usd FROM latest, prev
            "#,
        )
        .bind(protocol_id)
        .bind(window_secs)
        .fetch_optional(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        Ok(row.and_then(|r| {
            let latest: f64 = r.get(0);
            let prev: f64 = r.get(1);
            if prev <= 0.0 {
                None
            } else {
                Some((prev - latest) / prev * 100.0)
            }
        }))
    }
}
