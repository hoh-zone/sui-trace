use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use serde_json::{Value, json};
use trace_storage::repo::events::EventRepo;
use trace_storage::repo::packages::PackageRepo;
use trace_storage::repo::security::SecurityRepo;

use crate::state::AppState;

pub async fn get_package(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let repo = PackageRepo::new(&state.db);
    let pkg = repo
        .get(&id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let Some(pkg) = pkg else {
        return Err(StatusCode::NOT_FOUND);
    };
    let modules = repo.modules(&id).await.unwrap_or_default();
    let security = SecurityRepo::new(&state.db)
        .get_report(&id)
        .await
        .ok()
        .flatten();
    Ok(Json(json!({
        "package": pkg,
        "modules": modules,
        "security": security,
    })))
}

pub async fn get_security(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let report = SecurityRepo::new(&state.db)
        .get_report(&id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let Some(report) = report else {
        return Err(StatusCode::NOT_FOUND);
    };
    Ok(Json(json!({ "report": report })))
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
    let pkgs = PackageRepo::new(&state.db)
        .recent(limit)
        .await
        .unwrap_or_default();
    Json(json!({ "packages": pkgs }))
}

pub async fn events(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(q): Query<RecentQ>,
) -> Json<Value> {
    let events = EventRepo::new(&state.db)
        .list_by_package(&id, q.limit.clamp(1, 500))
        .await
        .unwrap_or_default();
    Json(json!({ "events": events }))
}
