//! Network-wide overview: latest checkpoint + 24h activity counters used to
//! drive the Home page header tiles.

use axum::{Json, extract::State};
use chrono::{Duration, Utc};
use serde_json::{Value, json};
use trace_storage::repo::checkpoints::CheckpointRepo;
use trace_storage::repo::packages::PackageRepo;
use trace_storage::repo::transactions::TransactionRepo;

use crate::state::AppState;

pub async fn overview(State(state): State<AppState>) -> Json<Value> {
    let day_ago = Utc::now() - Duration::hours(24);
    let cp = CheckpointRepo::new(&state.db)
        .latest()
        .await
        .unwrap_or_default();
    let tx_24h = TransactionRepo::new(&state.db)
        .count_since(day_ago)
        .await
        .unwrap_or_default();
    let pkg_24h = PackageRepo::new(&state.db)
        .count_since(day_ago)
        .await
        .unwrap_or_default();
    let pkg_total = PackageRepo::new(&state.db)
        .count()
        .await
        .unwrap_or_default();
    let tx_total = TransactionRepo::new(&state.db)
        .count()
        .await
        .unwrap_or_default();

    Json(json!({
        "checkpoint": cp,
        "tx_24h": tx_24h,
        "tx_total": tx_total,
        "packages_24h": pkg_24h,
        "packages_total": pkg_total,
    }))
}
