//! DefiLlama TVL poller.
//!
//! For each protocol registered in the `protocols` table with a non-null
//! `defillama_slug`, fetch `/protocol/{slug}` and persist the latest TVL
//! point. The full historical series is intentionally not re-fetched; we
//! rely on incremental snapshots accumulated locally.

use chrono::Utc;
use serde_json::Value;
use sqlx::Row;
use trace_common::{Error, error::Result, model::TvlPoint};
use trace_storage::Db;
use trace_storage::repo::tvl::TvlRepo;

pub struct TvlPoller {
    db: Db,
    base: String,
    http: reqwest::Client,
}

impl TvlPoller {
    pub fn new(db: Db, base: String) -> Self {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .expect("http client");
        Self { db, base, http }
    }

    pub async fn tick(&self) -> Result<()> {
        let rows = sqlx::query(
            r#"SELECT id, defillama_slug FROM protocols WHERE defillama_slug IS NOT NULL"#,
        )
        .fetch_all(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        let repo = TvlRepo::new(&self.db);
        for r in rows {
            let id: String = r.get(0);
            let slug: String = r.get(1);
            match self.fetch_protocol(&slug).await {
                Ok(point) => {
                    let p = TvlPoint {
                        protocol_id: id.clone(),
                        timestamp: Utc::now(),
                        tvl_usd: point,
                        breakdown: serde_json::json!({"slug": slug}),
                    };
                    if let Err(e) = repo.insert(&p).await {
                        tracing::warn!(protocol = %id, ?e, "failed to insert tvl point");
                    }
                }
                Err(e) => {
                    tracing::warn!(protocol = %id, slug, ?e, "failed to fetch tvl");
                }
            }
        }
        Ok(())
    }

    async fn fetch_protocol(&self, slug: &str) -> Result<f64> {
        let url = format!("{}/protocol/{}", self.base.trim_end_matches('/'), slug);
        let v: Value = self
            .http
            .get(url)
            .send()
            .await
            .map_err(|e| Error::Http(e.to_string()))?
            .json()
            .await
            .map_err(|e| Error::Http(e.to_string()))?;
        // DefiLlama returns either `tvl` (most recent) or a `currentChainTvls.Sui` map.
        let tvl = v
            .get("currentChainTvls")
            .and_then(|m| m.get("Sui"))
            .and_then(|x| x.as_f64())
            .or_else(|| v.get("tvl").and_then(|x| x.as_f64()))
            .unwrap_or(0.0);
        Ok(tvl)
    }
}
