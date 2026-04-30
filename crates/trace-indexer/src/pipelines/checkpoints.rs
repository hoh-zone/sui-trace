use async_trait::async_trait;
use trace_common::{error::Result, model::Checkpoint};
use trace_storage::Db;
use trace_storage::repo::checkpoints::CheckpointRepo;

use crate::model::CheckpointBundle;
use crate::pipeline::{Pipeline, PipelineKind};

pub struct CheckpointPipeline {
    db: Db,
}

impl CheckpointPipeline {
    pub fn new(db: Db) -> Self {
        Self { db }
    }
}

#[async_trait]
impl Pipeline for CheckpointPipeline {
    fn name(&self) -> &'static str {
        "checkpoints"
    }

    fn kind(&self) -> PipelineKind {
        PipelineKind::Sequential
    }

    async fn process(&self, bundle: &CheckpointBundle) -> Result<()> {
        let cp = Checkpoint {
            sequence_number: bundle.checkpoint.sequence_number,
            digest: bundle.checkpoint.digest.clone(),
            timestamp_ms: bundle.checkpoint.timestamp_ms,
            previous_digest: bundle.checkpoint.previous_digest.clone(),
            network_total_transactions: bundle.checkpoint.network_total_transactions,
            epoch: bundle.checkpoint.epoch,
        };
        CheckpointRepo::new(&self.db).upsert(&cp).await
    }
}
