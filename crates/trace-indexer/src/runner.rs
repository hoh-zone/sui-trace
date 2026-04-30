//! Pipeline orchestrator.
//!
//! The runner reads the high-watermark for every registered pipeline,
//! computes the lowest unprocessed checkpoint, and then walks the chain
//! forward in batches. Each checkpoint is fanned out to the registered
//! pipelines concurrently; sequential pipelines are awaited per checkpoint
//! before the watermark advances.

use std::sync::Arc;
use std::time::Duration;

use futures::future::join_all;
use trace_common::{Error, error::Result};
use trace_storage::Db;
use trace_storage::repo::checkpoints::CheckpointRepo;

use crate::client::SuiClient;
use crate::pipeline::{Pipeline, PipelineKind};

pub struct IndexerRunner {
    client: SuiClient,
    db: Db,
    pipelines: Vec<Arc<dyn Pipeline>>,
    /// Number of checkpoints to fetch per loop iteration.
    pub batch_size: usize,
    /// Minimum sequence to start from when no watermark exists.
    pub start_from: u64,
}

impl IndexerRunner {
    pub fn new(client: SuiClient, db: Db) -> Self {
        Self {
            client,
            db,
            pipelines: Vec::new(),
            batch_size: 25,
            start_from: 0,
        }
    }

    pub fn register<P: Pipeline + 'static>(&mut self, pipeline: P) -> &mut Self {
        self.pipelines.push(Arc::new(pipeline));
        self
    }

    pub fn pipelines(&self) -> &[Arc<dyn Pipeline>] {
        &self.pipelines
    }

    /// Run forever, advancing the watermarks. The function only returns on a
    /// fatal error (network or DB). Callers usually wrap it in a supervisor.
    pub async fn run(self) -> Result<()> {
        if self.pipelines.is_empty() {
            return Err(Error::Indexer("no pipelines registered".into()));
        }
        let cp_repo = CheckpointRepo::new(&self.db);
        let mut next_seq = self.compute_start(&cp_repo).await?;
        tracing::info!(start = next_seq, "indexer started");

        loop {
            let latest = match self.client.latest_checkpoint().await {
                Ok(v) => v,
                Err(e) => {
                    tracing::warn!(?e, "failed to fetch latest checkpoint, backing off");
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
            };

            if next_seq > latest {
                tokio::time::sleep(Duration::from_secs(2)).await;
                continue;
            }

            let end = (next_seq + self.batch_size as u64 - 1).min(latest);
            let mut seq = next_seq;
            while seq <= end {
                if let Err(e) = self.process_one(seq, &cp_repo).await {
                    tracing::error!(seq, ?e, "pipeline error, retrying after backoff");
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    break;
                }
                seq += 1;
                next_seq = seq;
            }
        }
    }

    async fn compute_start(&self, repo: &CheckpointRepo<'_>) -> Result<u64> {
        let mut start = self.start_from;
        for p in &self.pipelines {
            if let Some(wm) = repo.watermark(p.name()).await? {
                start = start.max(wm + 1);
            }
        }
        Ok(start)
    }

    async fn process_one(&self, seq: u64, repo: &CheckpointRepo<'_>) -> Result<()> {
        let bundle = match self.client.get_checkpoint(seq).await? {
            Some(b) => b,
            None => return Ok(()),
        };

        let (sequential, concurrent): (Vec<_>, Vec<_>) = self
            .pipelines
            .iter()
            .cloned()
            .partition(|p| p.kind() == PipelineKind::Sequential);

        for p in &sequential {
            p.process(&bundle).await?;
            repo.set_watermark(p.name(), seq).await?;
        }

        let concurrent_results = join_all(concurrent.iter().map(|p| {
            let p = p.clone();
            let bundle = bundle.clone();
            async move {
                let res = p.process(&bundle).await;
                (p.name(), res)
            }
        }))
        .await;

        for (name, res) in concurrent_results {
            res.map_err(|e| Error::Indexer(format!("pipeline {name} failed: {e}")))?;
            repo.set_watermark(name, seq).await?;
        }

        if seq.is_multiple_of(100) {
            tracing::info!(seq, "indexer progress");
        }
        Ok(())
    }
}
