use axum::{Json, extract::State};
use serde_json::{Value, json};
use trace_storage::repo::protocols::ProtocolRepo;

use crate::state::AppState;

pub async fn list(State(state): State<AppState>) -> Json<Value> {
    let protocols = ProtocolRepo::new(&state.db)
        .list(false)
        .await
        .unwrap_or_default();
    Json(json!({ "protocols": protocols }))
}
