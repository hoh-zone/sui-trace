use async_trait::async_trait;
use trace_common::{Error, error::Result, time::from_millis};
use trace_storage::Db;

use crate::model::CheckpointBundle;
use crate::pipeline::{Pipeline, PipelineKind};

pub struct ObjectPipeline {
    db: Db,
}

impl ObjectPipeline {
    pub fn new(db: Db) -> Self {
        Self { db }
    }
}

#[async_trait]
impl Pipeline for ObjectPipeline {
    fn name(&self) -> &'static str {
        "objects"
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
            for obj in &t.mutated_objects {
                let _ = sqlx::query(
                    r#"INSERT INTO objects (object_id, version, object_type, owner, contents, updated_at)
                       VALUES ($1, $2, $3, $4, $5, $6)
                       ON CONFLICT (object_id, version) DO NOTHING"#,
                )
                .bind(&obj.object_id)
                .bind(obj.version as i64)
                .bind(&obj.object_type)
                .bind(&obj.owner)
                .bind(&obj.contents)
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
