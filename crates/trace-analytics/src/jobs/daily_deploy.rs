use chrono::{Duration, Utc};
use trace_common::error::Result;
use trace_storage::Db;
use trace_storage::repo::analytics::AnalyticsRepo;

pub struct DailyDeployJob {
    db: Db,
}

impl DailyDeployJob {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    pub async fn run(&self) -> Result<()> {
        let to = Utc::now();
        let from = to - Duration::days(31);
        let stats = AnalyticsRepo::new(&self.db).daily_deploys(from, to).await?;
        tracing::info!(rows = stats.len(), "daily_deploys recomputed");
        Ok(())
    }
}
