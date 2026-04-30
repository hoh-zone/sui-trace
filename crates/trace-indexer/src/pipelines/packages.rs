//! Captures `Publish` and `Upgrade` operations.
//!
//! When a new package is observed it is also pushed to a Redis queue named
//! `trace:packages:to_scan`. The security worker subscribes to this queue and
//! triggers a full static analysis run.

use std::collections::HashMap;

use async_trait::async_trait;
use redis::AsyncCommands;
use serde_json::{Value, json};
use trace_common::{
    Error,
    error::Result,
    model::{Package, PackageModule},
    time::from_millis,
};
use trace_storage::repo::alerts::AlertRepo;
use trace_storage::repo::packages::PackageRepo;
use trace_storage::repo::protocols::ProtocolRepo;
use trace_storage::repo::source::SourceRepo;
use trace_storage::{Cache, Db};
use uuid::Uuid;

use crate::model::{CheckpointBundle, RawPackage};
use crate::pipeline::{Pipeline, PipelineKind};

pub struct PackagePipeline {
    db: Db,
    cache: Cache,
}

impl PackagePipeline {
    pub fn new(db: Db, cache: Cache) -> Self {
        Self { db, cache }
    }

    async fn enqueue_scan(&self, package_id: &str, version: u64) -> Result<()> {
        let payload = json!({ "package_id": package_id, "version": version }).to_string();
        let mut conn = self
            .cache
            .pool()
            .get()
            .await
            .map_err(|e| Error::Internal(e.to_string()))?;
        let _: i64 = conn
            .lpush("trace:packages:to_scan", payload)
            .await
            .map_err(|e| Error::Internal(e.to_string()))?;
        Ok(())
    }
}

#[async_trait]
impl Pipeline for PackagePipeline {
    fn name(&self) -> &'static str {
        "packages"
    }

    fn kind(&self) -> PipelineKind {
        PipelineKind::Sequential
    }

    async fn process(&self, bundle: &CheckpointBundle) -> Result<()> {
        let ts = from_millis(bundle.checkpoint.timestamp_ms);
        let repo = PackageRepo::new(&self.db);
        let source_repo = SourceRepo::new(&self.db);
        for tx in &bundle.transactions {
            for pkg in &tx.published_packages {
                let p = Package {
                    id: pkg.id.clone(),
                    original_id: pkg.original_id.clone(),
                    version: pkg.version,
                    publisher: pkg.publisher.clone(),
                    modules_count: pkg.modules.len() as u32,
                    source_verified: false,
                    published_at: ts,
                };
                let modules: Vec<PackageModule> = pkg
                    .modules
                    .iter()
                    .map(|m| PackageModule {
                        package_id: pkg.id.clone(),
                        module_name: m.name.clone(),
                        bytecode_hash: hash_bytecode(&m.bytecode_hex),
                        abi_json: m.abi.clone(),
                    })
                    .collect();
                repo.upsert(&p, &modules).await?;

                // Track upgrade lineage so the explorer can render the
                // publish → upgrade chain without recursive queries. The
                // publish/upgrade tx digest is captured here too so the
                // frontend can deep-link from the "Versions" tab.
                if let Err(e) = source_repo
                    .record_version(
                        &pkg.id,
                        &pkg.original_id,
                        pkg.version,
                        &pkg.publisher,
                        Some(tx.digest.as_str()),
                        ts,
                    )
                    .await
                {
                    tracing::warn!(?e, package = %pkg.id, "failed to record package_versions row");
                }

                // Fan-out to the curated watchlist: if this `original_id` is
                // listed under any protocol, record a `protocol_code_events`
                // row + an alert when the change looks risky.
                if let Err(e) = self.fan_out_to_watchlist(pkg, tx.digest.as_str(), ts).await {
                    tracing::warn!(?e, package = %pkg.id, "failed to fan-out protocol code event");
                }

                if let Err(e) = self.enqueue_scan(&pkg.id, pkg.version).await {
                    tracing::warn!(?e, package = %pkg.id, "failed to enqueue security scan");
                }
            }
        }
        Ok(())
    }
}

fn hash_bytecode(hex_str: &str) -> String {
    use sha2::{Digest, Sha256};
    let bytes = hex::decode(hex_str.trim_start_matches("0x")).unwrap_or_default();
    let hash = Sha256::digest(&bytes);
    hex::encode(hash)
}

impl PackagePipeline {
    /// Walk the curated protocol registry and, for every protocol whose
    /// `package_ids` contain this `original_id`, record a code-event row +
    /// emit a system-level alert when the change looks non-trivial.
    async fn fan_out_to_watchlist(
        &self,
        pkg: &RawPackage,
        publish_tx: &str,
        ts: chrono::DateTime<chrono::Utc>,
    ) -> Result<()> {
        let proto_repo = ProtocolRepo::new(&self.db);
        let protocols = proto_repo.protocols_for_original(&pkg.original_id).await?;
        if protocols.is_empty() {
            return Ok(());
        }

        // Resolve previous package_id for the lineage. The version we just
        // recorded above carries the right `previous_id` already.
        let prev_id = SourceRepo::new(&self.db)
            .version(&pkg.id)
            .await?
            .and_then(|v| v.previous_id);

        let kind = if prev_id.is_some() {
            "upgrade"
        } else {
            "publish"
        };
        let summary = build_summary(&self.db, prev_id.as_deref(), pkg).await;
        let severity = severity_for_summary(kind, &summary);

        let alert_repo = AlertRepo::new(&self.db);
        for proto in protocols {
            if let Err(e) = proto_repo
                .record_code_event(
                    &proto.id,
                    &pkg.id,
                    &pkg.original_id,
                    pkg.version,
                    prev_id.as_deref(),
                    Some(publish_tx),
                    &pkg.publisher,
                    kind,
                    &summary,
                    &severity,
                    ts,
                )
                .await
            {
                tracing::warn!(?e, protocol = %proto.id, "record_code_event failed");
                continue;
            }

            // Push an alert for anything beyond a routine info-level event
            // so operators see notable changes in the live feed.
            if severity != "info" {
                let payload = json!({
                    "protocol_id":  proto.id,
                    "protocol_name": proto.name,
                    "package_id":   pkg.id,
                    "version":      pkg.version,
                    "previous_id":  prev_id,
                    "kind":         kind,
                    "severity":     severity,
                    "summary":      summary,
                    "publish_tx":   publish_tx,
                    "publisher":    pkg.publisher,
                    "happened_at":  ts,
                });
                if let Err(e) = alert_repo
                    .record(
                        Uuid::new_v4(),
                        None,
                        None,
                        &format!("protocol.{kind}.{severity}"),
                        &payload,
                    )
                    .await
                {
                    tracing::warn!(?e, "failed to write protocol alert");
                }
            }
        }
        Ok(())
    }
}

/// Compare the new `RawPackage`'s modules against the modules stored for the
/// previous version (if any) and return a JSON summary describing the diff.
async fn build_summary(db: &Db, prev_id: Option<&str>, pkg: &RawPackage) -> Value {
    let mut new_hashes: HashMap<String, String> = HashMap::new();
    for m in &pkg.modules {
        new_hashes.insert(m.name.clone(), hash_bytecode(&m.bytecode_hex));
    }

    let mut prev_hashes: HashMap<String, String> = HashMap::new();
    if let Some(prev) = prev_id
        && let Ok(mods) = PackageRepo::new(db).modules(prev).await
    {
        for m in mods {
            prev_hashes.insert(m.module_name, m.bytecode_hash);
        }
    }

    let mut added = Vec::new();
    let mut changed = Vec::new();
    for (name, new_hash) in &new_hashes {
        match prev_hashes.get(name) {
            None => added.push(name.clone()),
            Some(prev_hash) if prev_hash != new_hash => {
                changed.push(json!({
                    "module": name,
                    "prev_hash": prev_hash,
                    "new_hash": new_hash,
                }));
            }
            _ => {}
        }
    }
    let removed: Vec<String> = prev_hashes
        .keys()
        .filter(|k| !new_hashes.contains_key(*k))
        .cloned()
        .collect();

    json!({
        "modules_total": new_hashes.len(),
        "modules_added": added,
        "modules_removed": removed,
        "modules_changed": changed,
    })
}

/// Map the diff onto a severity label used by the alert pipeline.
fn severity_for_summary(kind: &str, summary: &Value) -> String {
    if kind == "publish" {
        // First publish under a watched protocol — usually an addition of a
        // brand-new package_id by the same author. Worth surfacing as info.
        return "info".into();
    }
    let added = summary
        .get("modules_added")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    let removed = summary
        .get("modules_removed")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    let changed = summary
        .get("modules_changed")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    if removed > 0 {
        // Module removal on a live protocol is unusual and worth a louder
        // alarm.
        "critical".into()
    } else if added > 0 || changed > 1 {
        "warning".into()
    } else {
        "info".into()
    }
}
