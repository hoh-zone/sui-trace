use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::Deserialize;
use serde_json::{Value, json};
use trace_storage::repo::events::EventRepo;
use trace_storage::repo::transactions::TransactionRepo;

use crate::state::AppState;

#[derive(Deserialize)]
pub struct Pagination {
    #[serde(default = "limit_default")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}
fn limit_default() -> i64 {
    50
}

pub async fn get_address(State(state): State<AppState>, Path(addr): Path<String>) -> Json<Value> {
    let labels = state.labels.lookup(&addr).await.unwrap_or_default();
    let recent = TransactionRepo::new(&state.db)
        .list_by_address(&addr, 10, 0)
        .await
        .unwrap_or_default();
    Json(json!({
        "address": addr,
        "labels": labels,
        "recent_transactions": recent,
    }))
}

pub async fn transactions(
    State(state): State<AppState>,
    Path(addr): Path<String>,
    Query(p): Query<Pagination>,
) -> Json<Value> {
    let txs = TransactionRepo::new(&state.db)
        .list_by_address(&addr, p.limit.min(200), p.offset)
        .await
        .unwrap_or_default();
    Json(json!({ "transactions": txs }))
}

#[derive(Deserialize)]
pub struct EventQ {
    #[serde(default = "events_default_limit")]
    pub limit: i64,
}
fn events_default_limit() -> i64 {
    50
}

pub async fn events(
    State(state): State<AppState>,
    Path(addr): Path<String>,
    Query(q): Query<EventQ>,
) -> Json<Value> {
    let events = EventRepo::new(&state.db)
        .list_by_address(&addr, q.limit.clamp(1, 200))
        .await
        .unwrap_or_default();
    Json(json!({ "events": events }))
}
