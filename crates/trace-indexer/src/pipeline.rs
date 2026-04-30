//! Pipeline trait. A pipeline consumes one `CheckpointBundle` at a time and
//! writes its slice of state into the storage layer. The framework is
//! responsible for ordering, batching and watermarking; pipelines themselves
//! must be idempotent.

use async_trait::async_trait;
use trace_common::error::Result;

use crate::model::CheckpointBundle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineKind {
    /// Writes can happen out of order — checkpoints can be processed in
    /// parallel by multiple workers (events, balance changes).
    Concurrent,
    /// Writes must follow checkpoint order — the runner serialises invocations
    /// (packages, checkpoints).
    Sequential,
}

#[async_trait]
pub trait Pipeline: Send + Sync {
    /// Stable identifier used to persist the pipeline's high-watermark.
    fn name(&self) -> &'static str;

    fn kind(&self) -> PipelineKind;

    /// Process a single checkpoint. Implementations must be idempotent.
    async fn process(&self, bundle: &CheckpointBundle) -> Result<()>;
}
