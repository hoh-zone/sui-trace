use std::time::Duration;

use tokio::task::JoinSet;
use trace_common::error::Result;

use crate::jobs::{ActiveProjectsJob, DailyDeployJob};
use crate::tvl::TvlPoller;
use trace_storage::Db;

pub struct Scheduler {
    db: Db,
    defillama_base: String,
    defillama_interval_secs: u64,
}

impl Scheduler {
    pub fn new(db: Db, defillama_base: String, defillama_interval_secs: u64) -> Self {
        Self {
            db,
            defillama_base,
            defillama_interval_secs,
        }
    }

    pub async fn run(self) -> Result<()> {
        let mut set = JoinSet::new();

        // Daily deploy aggregation — every 30 minutes (cheap idempotent recompute).
        let db = self.db.clone();
        set.spawn(async move {
            let job = DailyDeployJob::new(db);
            loop {
                if let Err(e) = job.run().await {
                    tracing::warn!(?e, "daily deploy job failed");
                }
                tokio::time::sleep(Duration::from_secs(30 * 60)).await;
            }
        });

        // Active projects ranking — every 10 minutes.
        let db = self.db.clone();
        set.spawn(async move {
            let job = ActiveProjectsJob::new(db);
            loop {
                if let Err(e) = job.run().await {
                    tracing::warn!(?e, "active projects job failed");
                }
                tokio::time::sleep(Duration::from_secs(10 * 60)).await;
            }
        });

        // TVL poller.
        let db = self.db.clone();
        let base = self.defillama_base.clone();
        let interval = self.defillama_interval_secs;
        set.spawn(async move {
            let poller = TvlPoller::new(db, base);
            loop {
                if let Err(e) = poller.tick().await {
                    tracing::warn!(?e, "tvl poller failed");
                }
                tokio::time::sleep(Duration::from_secs(interval)).await;
            }
        });

        while let Some(res) = set.join_next().await {
            // Spawned tasks loop forever, so on this branch the join result is
            // always `Err(JoinError)` (the inner `Ok` variant is uninhabited).
            let Err(e) = res;
            tracing::error!(?e, "analytics task crashed");
        }
        Ok(())
    }
}
