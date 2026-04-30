use anyhow::Result;
use trace_analytics::Scheduler;
use trace_common::config::AppConfig;
use trace_storage::Db;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    trace_common::telemetry::init("trace-analytics");
    let cfg_path = std::env::var("TRACE_CONFIG").unwrap_or_else(|_| "config/default.toml".into());
    let cfg = AppConfig::load(&cfg_path)?;
    let db = Db::connect(&cfg.database).await?;
    let sched = Scheduler::new(
        db,
        cfg.defillama.base.clone(),
        cfg.defillama.poll_interval_secs,
    );
    sched.run().await?;
    Ok(())
}
