use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use serde_json::{Value, json};
use trace_common::model::AddressLabel;
use trace_labels::LabelService;

use crate::auth::AuthUser;
use crate::state::AppState;

pub async fn for_address(State(state): State<AppState>, Path(addr): Path<String>) -> Json<Value> {
    let labels = state.labels.lookup(&addr).await.unwrap_or_default();
    Json(json!({ "address": addr, "labels": labels }))
}

#[derive(Deserialize)]
pub struct SearchQ {
    pub q: String,
    #[serde(default = "default_limit")]
    pub limit: i64,
}
fn default_limit() -> i64 {
    25
}

pub async fn search(State(state): State<AppState>, Query(q): Query<SearchQ>) -> Json<Value> {
    let labels = state
        .labels
        .search(&q.q, q.limit.min(200))
        .await
        .unwrap_or_default();
    Json(json!({ "labels": labels }))
}

#[derive(Deserialize)]
pub struct SubmitBody {
    pub address: String,
    pub label: String,
    pub category: String,
    #[serde(default)]
    pub evidence_url: Option<String>,
}

pub async fn submit(
    State(state): State<AppState>,
    AuthUser(_claims): AuthUser,
    Json(body): Json<SubmitBody>,
) -> Result<Json<Value>, StatusCode> {
    let category = LabelService::parse_category(&body.category).ok_or(StatusCode::BAD_REQUEST)?;
    let label = AddressLabel {
        address: body.address.clone(),
        label: body.label.clone(),
        category,
        source: trace_common::model::LabelSource::Community,
        confidence: 0.4,
        evidence_url: body.evidence_url.clone(),
        verified: false,
    };
    state
        .labels
        .submit(label.clone())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(json!({ "label": label })))
}
