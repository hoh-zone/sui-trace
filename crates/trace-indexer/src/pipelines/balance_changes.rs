use async_trait::async_trait;
use std::str::FromStr;
use trace_common::{Error, error::Result, time::from_millis};
use trace_storage::Db;

use crate::model::CheckpointBundle;
use crate::pipeline::{Pipeline, PipelineKind};

pub struct BalanceChangePipeline {
    db: Db,
}

impl BalanceChangePipeline {
    pub fn new(db: Db) -> Self {
        Self { db }
    }
}

#[async_trait]
impl Pipeline for BalanceChangePipeline {
    fn name(&self) -> &'static str {
        "balance_changes"
    }

    fn kind(&self) -> PipelineKind {
        PipelineKind::Concurrent
    }

    async fn process(&self, bundle: &CheckpointBundle) -> Result<()> {
        let ts = from_millis(bundle.checkpoint.timestamp_ms);
        let mut tx = self
            .db
            .pool()
            .begin()
            .await
            .map_err(|e| Error::Database(e.to_string()))?;
        for t in &bundle.transactions {
            for c in &t.balance_changes {
                let amount = bigdecimal::BigDecimal::from_str(&c.amount)
                    .unwrap_or_else(|_| bigdecimal::BigDecimal::from(0));
                let _ = sqlx::query(
                    r#"INSERT INTO balance_changes (tx_digest, owner, coin_type, amount, timestamp)
                       VALUES ($1, $2, $3, $4, $5)"#,
                )
                .bind(&t.digest)
                .bind(&c.owner)
                .bind(&c.coin_type)
                .bind(&amount)
                .bind(ts)
                .execute(&mut *tx)
                .await
                .map_err(|e: sqlx::Error| Error::Database(e.to_string()))?;
            }
        }
        tx.commit()
            .await
            .map_err(|e| Error::Database(e.to_string()))?;
        Ok(())
    }
}
