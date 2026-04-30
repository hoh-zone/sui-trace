use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use serde_json::{Value, json};
use trace_storage::repo::{checkpoints::CheckpointRepo, transactions::TransactionRepo};

use crate::state::AppState;

pub async fn get_checkpoint(
    State(state): State<AppState>,
    Path(seq): Path<u64>,
) -> Result<Json<Value>, StatusCode> {
    let cp = CheckpointRepo::new(&state.db)
        .get(seq)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let Some(cp) = cp else {
        return Err(StatusCode::NOT_FOUND);
    };
    Ok(Json(json!({ "checkpoint": cp })))
}

pub async fn latest(State(state): State<AppState>) -> Json<Value> {
    let cp = CheckpointRepo::new(&state.db)
        .latest()
        .await
        .unwrap_or_default();
    Json(json!({ "checkpoint": cp }))
}

#[derive(Deserialize)]
pub struct RecentQ {
    #[serde(default = "default_recent_limit")]
    pub limit: i64,
}
fn default_recent_limit() -> i64 {
    25
}

pub async fn recent(State(state): State<AppState>, Query(q): Query<RecentQ>) -> Json<Value> {
    let limit = q.limit.clamp(1, 200);
    let cps = CheckpointRepo::new(&state.db)
        .recent(limit)
        .await
        .unwrap_or_default();
    Json(json!({ "checkpoints": cps }))
}

#[derive(Deserialize)]
pub struct CpTxQ {
    #[serde(default = "default_cp_tx_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}
fn default_cp_tx_limit() -> i64 {
    50
}

pub async fn transactions(
    State(state): State<AppState>,
    Path(seq): Path<u64>,
    Query(q): Query<CpTxQ>,
) -> Json<Value> {
    let txs = TransactionRepo::new(&state.db)
        .list_by_checkpoint(seq, q.limit.clamp(1, 200), q.offset.max(0))
        .await
        .unwrap_or_default();
    Json(json!({ "transactions": txs }))
}
