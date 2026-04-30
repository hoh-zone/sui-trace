use async_trait::async_trait;
use trace_common::{
    error::Result,
    model::{Transaction, TxStatus},
    time::from_millis,
};
use trace_storage::Db;
use trace_storage::repo::transactions::TransactionRepo;

use crate::model::CheckpointBundle;
use crate::pipeline::{Pipeline, PipelineKind};

pub struct TransactionPipeline {
    db: Db,
}

impl TransactionPipeline {
    pub fn new(db: Db) -> Self {
        Self { db }
    }
}

#[async_trait]
impl Pipeline for TransactionPipeline {
    fn name(&self) -> &'static str {
        "transactions"
    }

    fn kind(&self) -> PipelineKind {
        PipelineKind::Concurrent
    }

    async fn process(&self, bundle: &CheckpointBundle) -> Result<()> {
        let ts = from_millis(bundle.checkpoint.timestamp_ms);
        let txs: Vec<Transaction> = bundle
            .transactions
            .iter()
            .map(|t| Transaction {
                digest: t.digest.clone(),
                checkpoint_seq: bundle.checkpoint.sequence_number,
                timestamp: ts,
                sender: t.sender.clone(),
                status: if t.status == "success" {
                    TxStatus::Success
                } else {
                    TxStatus::Failure
                },
                gas_used: t.gas_used,
                gas_price: t.gas_price,
                kind: t.kind.clone(),
            })
            .collect();
        TransactionRepo::new(&self.db).upsert_many(&txs).await?;
        Ok(())
    }
}
