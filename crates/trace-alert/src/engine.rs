//! Alert engine entry point. Designed to be embedded inside a long-running
//! task that ticks the rule set and dispatches matching events.

use std::time::Duration;

use serde_json::Value;
use sha2::{Digest, Sha256};
use trace_common::{config::AlertConfig, error::Result};
use trace_storage::Db;
use trace_storage::repo::alerts::AlertRepo;
use uuid::Uuid;

use crate::channels::{self, Channel, ChannelDefaults, ChannelSpec};
use crate::dedup;
use crate::rules;

#[derive(Clone)]
pub struct AlertEngine {
    db: Db,
    cfg: AlertConfig,
    defaults: ChannelDefaults,
}

impl AlertEngine {
    pub fn new(db: Db, cfg: AlertConfig, defaults: ChannelDefaults) -> Self {
        Self { db, cfg, defaults }
    }

    pub async fn run(self) -> Result<()> {
        tracing::info!("alert engine started");
        loop {
            if let Err(e) = self.tick().await {
                tracing::warn!(?e, "alert tick failed");
            }
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    }

    pub async fn tick(&self) -> Result<()> {
        // Built-in global rules.
        let mut payloads: Vec<Value> = Vec::new();
        payloads.extend(rules::high_severity_package(&self.db, 60 * 60, 7.5).await?);
        payloads.extend(rules::large_outflow(&self.db, 100_000_000_000, 5 * 60).await?);
        payloads.extend(rules::suspicious_recipient(&self.db, 5 * 60).await?);
        payloads.extend(rules::package_upgrade(&self.db, 10 * 60).await?);

        for payload in payloads {
            self.fire(None, None, "global", &payload, &[]).await?;
        }
        Ok(())
    }

    /// Persist an alert + dispatch it through the requested channels.
    pub async fn fire(
        &self,
        user_id: Option<Uuid>,
        watchlist_id: Option<Uuid>,
        rule_id: &str,
        payload: &Value,
        channels_spec: &[ChannelSpec],
    ) -> Result<()> {
        let key = dedup_key(rule_id, payload);
        if !dedup::try_acquire(&self.db, &key, self.cfg.dedupe_window_secs as i64).await? {
            return Ok(());
        }
        let id = Uuid::new_v4();
        let repo = AlertRepo::new(&self.db);
        repo.record(id, user_id, watchlist_id, rule_id, payload)
            .await?;

        let mut delivered = false;
        for spec in channels_spec.iter().cloned() {
            let ch = channels::build(spec, &self.defaults);
            if let Err(e) = self.deliver(&*ch, payload).await {
                tracing::warn!(channel = ch.kind(), ?e, "delivery failed");
                let _ = repo.mark_failed(id, &format!("{}: {e}", ch.kind())).await;
            } else {
                delivered = true;
            }
        }
        if delivered || channels_spec.is_empty() {
            let _ = repo.mark_delivered(id).await;
        }
        Ok(())
    }

    async fn deliver(&self, ch: &dyn Channel, payload: &Value) -> Result<()> {
        let mut last_err = None;
        for attempt in 0..self.cfg.max_retries {
            match ch.send(payload).await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    last_err = Some(e);
                    let backoff = Duration::from_millis(200 * (attempt as u64 + 1).pow(2) * 100);
                    tokio::time::sleep(backoff).await;
                }
            }
        }
        Err(last_err
            .unwrap_or_else(|| trace_common::Error::Alert("delivery exhausted retries".into())))
    }
}

fn dedup_key(rule_id: &str, payload: &Value) -> String {
    let mut hasher = Sha256::new();
    hasher.update(rule_id.as_bytes());
    if let Ok(b) = serde_json::to_vec(payload) {
        hasher.update(&b);
    }
    format!("{rule_id}:{}", hex::encode(&hasher.finalize()[..8]))
}
