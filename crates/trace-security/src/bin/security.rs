use anyhow::Result;
use trace_common::config::AppConfig;
use trace_security::worker::SecurityWorker;
use trace_storage::{Cache, Db};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    trace_common::telemetry::init("trace-security");
    let cfg_path = std::env::var("TRACE_CONFIG").unwrap_or_else(|_| "config/default.toml".into());
    let cfg = AppConfig::load(&cfg_path)?;
    let db = Db::connect(&cfg.database).await?;
    let cache = Cache::connect(&cfg.redis)?;
    let worker = SecurityWorker::new(db, cache, cfg.sui.rest_url.clone());
    worker.run().await?;
    Ok(())
}
