use chrono::{DateTime, Utc};
use sqlx::Row;
use trace_common::{
    Error,
    error::Result,
    model::{Transaction, TxStatus},
};

use crate::Db;

pub struct TransactionRepo<'a> {
    db: &'a Db,
}

impl<'a> TransactionRepo<'a> {
    pub fn new(db: &'a Db) -> Self {
        Self { db }
    }

    pub async fn upsert_many(&self, txs: &[Transaction]) -> Result<u64> {
        if txs.is_empty() {
            return Ok(0);
        }
        let mut tx = self
            .db
            .pool()
            .begin()
            .await
            .map_err(|e| Error::Database(e.to_string()))?;
        let mut count = 0u64;
        for t in txs {
            let status = match t.status {
                TxStatus::Success => "success",
                TxStatus::Failure => "failure",
            };
            sqlx::query(
                r#"
                INSERT INTO transactions
                    (digest, checkpoint_seq, timestamp, sender, status, gas_used, gas_price, kind)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT (digest) DO NOTHING
                "#,
            )
            .bind(&t.digest)
            .bind(t.checkpoint_seq as i64)
            .bind(t.timestamp)
            .bind(&t.sender)
            .bind(status)
            .bind(t.gas_used as i64)
            .bind(t.gas_price as i64)
            .bind(&t.kind)
            .execute(&mut *tx)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;
            count += 1;
        }
        tx.commit()
            .await
            .map_err(|e| Error::Database(e.to_string()))?;
        Ok(count)
    }

    pub async fn get(&self, digest: &str) -> Result<Option<Transaction>> {
        let row = sqlx::query(
            r#"SELECT digest, checkpoint_seq, timestamp, sender, status, gas_used, gas_price, kind
               FROM transactions WHERE digest = $1"#,
        )
        .bind(digest)
        .fetch_optional(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(row.map(row_to_tx))
    }

    pub async fn list_by_address(
        &self,
        address: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Transaction>> {
        let rows = sqlx::query(
            r#"SELECT digest, checkpoint_seq, timestamp, sender, status, gas_used, gas_price, kind
               FROM transactions WHERE sender = $1
               ORDER BY timestamp DESC LIMIT $2 OFFSET $3"#,
        )
        .bind(address)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(rows.into_iter().map(row_to_tx).collect())
    }

    pub async fn latest(&self, limit: i64) -> Result<Vec<Transaction>> {
        let rows = sqlx::query(
            r#"SELECT digest, checkpoint_seq, timestamp, sender, status, gas_used, gas_price, kind
               FROM transactions ORDER BY timestamp DESC LIMIT $1"#,
        )
        .bind(limit)
        .fetch_all(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(rows.into_iter().map(row_to_tx).collect())
    }

    pub async fn list_by_checkpoint(
        &self,
        seq: u64,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Transaction>> {
        let rows = sqlx::query(
            r#"SELECT digest, checkpoint_seq, timestamp, sender, status, gas_used, gas_price, kind
               FROM transactions WHERE checkpoint_seq = $1
               ORDER BY timestamp DESC LIMIT $2 OFFSET $3"#,
        )
        .bind(seq as i64)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(rows.into_iter().map(row_to_tx).collect())
    }

    pub async fn count(&self) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*)::bigint FROM transactions")
            .fetch_one(self.db.pool())
            .await
            .map_err(|e| Error::Database(e.to_string()))?;
        Ok(row.get(0))
    }

    /// Count of transactions whose `timestamp` is strictly greater than `since`.
    pub async fn count_since(&self, since: DateTime<Utc>) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*)::bigint FROM transactions WHERE timestamp > $1")
            .bind(since)
            .fetch_one(self.db.pool())
            .await
            .map_err(|e| Error::Database(e.to_string()))?;
        Ok(row.get(0))
    }
}

fn row_to_tx(r: sqlx::postgres::PgRow) -> Transaction {
    let status_s: String = r.get(4);
    let status = match status_s.as_str() {
        "success" => TxStatus::Success,
        _ => TxStatus::Failure,
    };
    Transaction {
        digest: r.get(0),
        checkpoint_seq: r.get::<i64, _>(1) as u64,
        timestamp: r.get::<DateTime<Utc>, _>(2),
        sender: r.get(3),
        status,
        gas_used: r.get::<i64, _>(5) as u64,
        gas_price: r.get::<i64, _>(6) as u64,
        kind: r.get(7),
    }
}
