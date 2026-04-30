use axum::{Json, extract::State, http::StatusCode};
use serde_json::{Value, json};

use crate::state::AppState;

pub async fn health(State(state): State<AppState>) -> (StatusCode, Json<Value>) {
    let db_ok = state.db.health().await.is_ok();
    let body = json!({
        "status": if db_ok { "ok" } else { "degraded" },
        "service": "trace-api",
        "version": env!("CARGO_PKG_VERSION"),
        "db": db_ok,
    });
    let code = if db_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (code, Json(body))
}
