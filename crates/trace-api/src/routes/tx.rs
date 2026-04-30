use std::collections::BTreeSet;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use redis::AsyncCommands;
use serde::Deserialize;
use serde_json::{Value, json};
use trace_storage::repo::events::EventRepo;
use trace_storage::repo::labels::LabelRepo;
use trace_storage::repo::security::SecurityRepo;
use trace_storage::repo::transactions::TransactionRepo;

use crate::state::AppState;

pub async fn get_tx(
    State(state): State<AppState>,
    Path(digest): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let tx = TransactionRepo::new(&state.db)
        .get(&digest)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let Some(tx) = tx else {
        return Err(StatusCode::NOT_FOUND);
    };
    let events = EventRepo::new(&state.db)
        .list_by_tx(&digest)
        .await
        .unwrap_or_default();
    Ok(Json(json!({"transaction": tx, "events": events})))
}

#[derive(Deserialize)]
pub struct LatestQ {
    #[serde(default = "default_limit")]
    pub limit: i64,
}
fn default_limit() -> i64 {
    25
}

pub async fn latest(State(state): State<AppState>, Query(q): Query<LatestQ>) -> Json<Value> {
    let txs = TransactionRepo::new(&state.db)
        .latest(q.limit.min(200))
        .await
        .unwrap_or_default();
    Json(json!({ "transactions": txs }))
}

/// SuiVision-style "full transaction" detail. Combines:
///
/// * The indexed `transactions` row from our DB (so we always know the
///   checkpoint sequence, even when the RPC fullnode pruned the tx).
/// * The full `sui_getTransactionBlock` payload fetched live from the Sui
///   JSON-RPC fullnode, with `showInput`, `showRawInput`, `showEffects`,
///   `showEvents`, `showBalanceChanges` and `showObjectChanges` all set.
///   The result is cached in Redis for an hour because finalized tx data
///   is immutable.
/// * Address labels for the sender and every owner appearing in
///   `balanceChanges` / `objectChanges`.
/// * Security report summaries for every package referenced by a
///   `MoveCall` command (so the UI can flag risky calls inline).
/// * Indexed events for the tx (so the page works even when the RPC has
///   pruned events).
pub async fn full(
    State(state): State<AppState>,
    Path(digest): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    // ---- 1. Indexed metadata --------------------------------------------
    let indexed = TransactionRepo::new(&state.db)
        .get(&digest)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let events = EventRepo::new(&state.db)
        .list_by_tx(&digest)
        .await
        .unwrap_or_default();

    // ---- 2. Live RPC fetch with Redis cache -----------------------------
    let cache_key = format!("trace:tx:full:{digest}");
    let mut redis = state.cache.pool().get().await.ok();

    let mut rpc: Option<Value> = None;
    if let Some(conn) = redis.as_mut()
        && let Ok(Some(s)) = conn.get::<_, Option<String>>(&cache_key).await
        && let Ok(v) = serde_json::from_str::<Value>(&s)
    {
        rpc = Some(v);
    }

    if rpc.is_none() {
        match state.sui.get_transaction_block(&digest).await {
            Ok(v) => {
                if let Some(conn) = redis.as_mut()
                    && let Ok(s) = serde_json::to_string(&v)
                {
                    // Cache for 1h; finalized Sui tx data is immutable.
                    let _: redis::RedisResult<()> = conn.set_ex(&cache_key, s, 3600).await;
                }
                rpc = Some(v);
            }
            Err(e) => {
                tracing::warn!(?e, %digest, "sui_getTransactionBlock failed");
            }
        }
    }

    // ---- 3. Enrichment: addresses + packages ----------------------------
    let mut addresses: BTreeSet<String> = BTreeSet::new();
    let mut packages: BTreeSet<String> = BTreeSet::new();
    if let Some(t) = indexed.as_ref() {
        addresses.insert(t.sender.clone());
    }
    if let Some(rpc) = rpc.as_ref() {
        collect_addresses_and_packages(rpc, &mut addresses, &mut packages);
    }

    let labels_repo = LabelRepo::new(&state.db);
    let mut labels_map = serde_json::Map::new();
    for addr in &addresses {
        if let Ok(lbls) = labels_repo.for_address(addr).await
            && !lbls.is_empty()
        {
            labels_map.insert(
                addr.clone(),
                serde_json::to_value(lbls).unwrap_or(Value::Null),
            );
        }
    }

    let sec_repo = SecurityRepo::new(&state.db);
    let mut packages_map = serde_json::Map::new();
    for pkg in &packages {
        if let Ok(Some(report)) = sec_repo.get_report(pkg).await {
            packages_map.insert(
                pkg.clone(),
                json!({
                    "score": report.score,
                    "max_severity": report.max_severity,
                    "findings_count": report.findings.len(),
                    "scanned_at": report.scanned_at,
                }),
            );
        }
    }

    Ok(Json(json!({
        "digest":        digest,
        "indexed":       indexed,        // null when fullnode is the only source
        "events":        events,         // indexed events (always present when we ingested it)
        "rpc":           rpc,            // raw sui_getTransactionBlock payload
        "labels":        Value::Object(labels_map),
        "packages":      Value::Object(packages_map),
    })))
}

fn collect_addresses_and_packages(
    v: &Value,
    addrs: &mut BTreeSet<String>,
    pkgs: &mut BTreeSet<String>,
) {
    // Sender
    if let Some(s) = v
        .pointer("/transaction/data/sender")
        .and_then(Value::as_str)
    {
        addrs.insert(s.to_string());
    }

    // Gas owner
    if let Some(s) = v
        .pointer("/transaction/data/gasData/owner")
        .and_then(Value::as_str)
    {
        addrs.insert(s.to_string());
    }

    // Balance changes / object changes owners
    for key in ["balanceChanges", "objectChanges"] {
        if let Some(arr) = v.get(key).and_then(Value::as_array) {
            for c in arr {
                if let Some(o) = c.get("owner") {
                    extract_owner_into(o, addrs);
                }
                // recipient (TransferObjects-style hints in objectChanges)
                if let Some(recipient) = c.get("recipient") {
                    extract_owner_into(recipient, addrs);
                }
                if let Some(s) = c.get("packageId").and_then(Value::as_str) {
                    pkgs.insert(s.to_string());
                }
            }
        }
    }

    // Programmable tx commands carry MoveCall.package
    if let Some(arr) = v
        .pointer("/transaction/data/transaction/transactions")
        .and_then(Value::as_array)
    {
        for cmd in arr {
            if let Some(call) = cmd.get("MoveCall")
                && let Some(p) = call.get("package").and_then(Value::as_str)
            {
                pkgs.insert(p.to_string());
            }
            if let Some(p) = cmd.pointer("/Publish") {
                // result of Publish is a new package id but it's in
                // objectChanges — already collected.
                let _ = p;
            }
        }
    }

    // Effects.created/mutated/deleted owners (older RPC shape)
    if let Some(eff) = v.get("effects") {
        for key in ["created", "mutated", "unwrapped", "deleted", "wrapped"] {
            if let Some(arr) = eff.get(key).and_then(Value::as_array) {
                for c in arr {
                    if let Some(o) = c.get("owner") {
                        extract_owner_into(o, addrs);
                    }
                }
            }
        }
    }
}

fn extract_owner_into(v: &Value, addrs: &mut BTreeSet<String>) {
    match v {
        Value::String(s) => {
            addrs.insert(s.clone());
        }
        Value::Object(map) => {
            for k in ["AddressOwner", "ObjectOwner"] {
                if let Some(s) = map.get(k).and_then(Value::as_str) {
                    addrs.insert(s.to_string());
                }
            }
            // `Shared { initial_shared_version }` and `Immutable` skipped.
        }
        _ => {}
    }
}
