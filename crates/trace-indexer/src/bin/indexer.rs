use anyhow::Result;
use trace_common::config::AppConfig;
use trace_indexer::IndexerRunner;
use trace_indexer::client::SuiClient;
use trace_indexer::pipelines::{
    BalanceChangePipeline, CheckpointPipeline, EventPipeline, ObjectPipeline, PackagePipeline,
    TransactionPipeline,
};
use trace_storage::{Cache, Db};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    trace_common::telemetry::init("trace-indexer");

    let cfg_path = std::env::var("TRACE_CONFIG").unwrap_or_else(|_| "config/default.toml".into());
    let cfg = AppConfig::load(&cfg_path)?;

    let db = Db::connect(&cfg.database).await?;
    db.migrate().await?;
    let cache = Cache::connect(&cfg.redis)?;

    let client = SuiClient::new(&cfg.sui.rest_url);
    let mut runner = IndexerRunner::new(client, db.clone());
    runner.batch_size = cfg.indexer.batch_size.max(10);
    runner.start_from = cfg.sui.start_checkpoint;

    runner.register(CheckpointPipeline::new(db.clone()));
    runner.register(TransactionPipeline::new(db.clone()));
    runner.register(EventPipeline::new(db.clone()));
    runner.register(ObjectPipeline::new(db.clone()));
    runner.register(BalanceChangePipeline::new(db.clone()));
    runner.register(PackagePipeline::new(db.clone(), cache));

    runner.run().await?;
    Ok(())
}
