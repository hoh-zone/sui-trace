use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::state::AppState;

#[derive(Deserialize)]
pub struct SearchQ {
    pub q: String,
}

pub async fn search(State(state): State<AppState>, Query(q): Query<SearchQ>) -> Json<Value> {
    let term = q.q.trim();
    let kind = classify(term);
    let labels = state.labels.search(term, 10).await.unwrap_or_default();
    Json(json!({
        "query": term,
        "kind": kind,
        "labels": labels,
    }))
}

fn classify(q: &str) -> &'static str {
    if !q.starts_with("0x") {
        return "term";
    }
    match q.len() {
        66 => "address_or_object",
        64 | 44 | 46 => "digest",
        _ => "unknown",
    }
}
