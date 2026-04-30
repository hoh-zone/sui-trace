use chrono::{DateTime, NaiveDate, Utc};
use sqlx::Row;
use trace_common::{Error, error::Result};

use crate::Db;

#[derive(Debug, Clone)]
pub struct DailyDeployStat {
    pub day: NaiveDate,
    pub package_count: i64,
    pub unique_publishers: i64,
}

#[derive(Debug, Clone)]
pub struct PackageRanking {
    pub package_id: String,
    pub calls: i64,
    pub unique_callers: i64,
    pub gas_total: i64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ThroughputPoint {
    pub bucket: DateTime<Utc>,
    pub tx_count: i64,
}

pub struct AnalyticsRepo<'a> {
    db: &'a Db,
}

impl<'a> AnalyticsRepo<'a> {
    pub fn new(db: &'a Db) -> Self {
        Self { db }
    }

    pub async fn daily_deploys(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<DailyDeployStat>> {
        let rows = sqlx::query(
            r#"
            SELECT date_trunc('day', published_at)::date AS day,
                   COUNT(*)::bigint AS package_count,
                   COUNT(DISTINCT publisher)::bigint AS unique_publishers
            FROM packages
            WHERE published_at >= $1 AND published_at < $2
            GROUP BY day
            ORDER BY day
            "#,
        )
        .bind(from)
        .bind(to)
        .fetch_all(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(rows
            .into_iter()
            .map(|r| DailyDeployStat {
                day: r.get(0),
                package_count: r.get(1),
                unique_publishers: r.get(2),
            })
            .collect())
    }

    /// Per-minute transaction throughput across the last `minutes` minutes,
    /// returned ascending. Useful for the live throughput sparkline.
    pub async fn tx_throughput(&self, minutes: i64) -> Result<Vec<ThroughputPoint>> {
        let rows = sqlx::query(
            r#"
            SELECT date_trunc('minute', timestamp) AS bucket,
                   COUNT(*)::bigint AS tx_count
            FROM transactions
            WHERE timestamp >= NOW() - ($1 || ' minutes')::interval
            GROUP BY bucket
            ORDER BY bucket
            "#,
        )
        .bind(minutes.to_string())
        .fetch_all(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(rows
            .into_iter()
            .map(|r| ThroughputPoint {
                bucket: r.get(0),
                tx_count: r.get(1),
            })
            .collect())
    }

    pub async fn active_packages(
        &self,
        since: DateTime<Utc>,
        limit: i64,
    ) -> Result<Vec<PackageRanking>> {
        let rows = sqlx::query(
            r#"
            SELECT package_id,
                   COUNT(*)::bigint AS calls,
                   COUNT(DISTINCT sender)::bigint AS unique_callers,
                   COALESCE(SUM(gas_used), 0)::bigint AS gas_total
            FROM events e
            JOIN transactions t ON t.digest = e.tx_digest
            WHERE e.timestamp >= $1
            GROUP BY package_id
            ORDER BY calls DESC
            LIMIT $2
            "#,
        )
        .bind(since)
        .bind(limit)
        .fetch_all(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(rows
            .into_iter()
            .map(|r| PackageRanking {
                package_id: r.get(0),
                calls: r.get(1),
                unique_callers: r.get(2),
                gas_total: r.get(3),
            })
            .collect())
    }
}
