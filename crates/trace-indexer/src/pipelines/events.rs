use async_trait::async_trait;
use trace_common::{error::Result, model::Event, time::from_millis};
use trace_storage::Db;
use trace_storage::repo::events::EventRepo;

use crate::model::CheckpointBundle;
use crate::pipeline::{Pipeline, PipelineKind};

pub struct EventPipeline {
    db: Db,
}

impl EventPipeline {
    pub fn new(db: Db) -> Self {
        Self { db }
    }
}

#[async_trait]
impl Pipeline for EventPipeline {
    fn name(&self) -> &'static str {
        "events"
    }

    fn kind(&self) -> PipelineKind {
        PipelineKind::Concurrent
    }

    async fn process(&self, bundle: &CheckpointBundle) -> Result<()> {
        let ts = from_millis(bundle.checkpoint.timestamp_ms);
        let mut events = Vec::new();
        for tx in &bundle.transactions {
            for ev in &tx.events {
                events.push(Event {
                    tx_digest: tx.digest.clone(),
                    event_seq: ev.seq,
                    package_id: ev.package_id.clone(),
                    module: ev.module.clone(),
                    event_type: ev.event_type.clone(),
                    sender: ev.sender.clone(),
                    timestamp: ts,
                    payload: ev.payload.clone(),
                });
            }
        }
        EventRepo::new(&self.db).insert_many(&events).await?;
        Ok(())
    }
}
