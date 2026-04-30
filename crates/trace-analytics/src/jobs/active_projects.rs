use chrono::{Duration, Utc};
use trace_common::error::Result;
use trace_storage::Db;
use trace_storage::repo::analytics::AnalyticsRepo;

pub struct ActiveProjectsJob {
    db: Db,
}

impl ActiveProjectsJob {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    pub async fn run(&self) -> Result<()> {
        let since = Utc::now() - Duration::hours(24);
        let rankings = AnalyticsRepo::new(&self.db)
            .active_packages(since, 100)
            .await?;
        tracing::info!(count = rankings.len(), "active_projects refreshed");
        Ok(())
    }
}
