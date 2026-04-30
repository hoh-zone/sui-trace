//! Global security endpoints — recent findings feed, severity scoreboard
//! and rule rankings. The per-package report still lives in `packages.rs`.

use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;
use serde_json::{Value, json};
use trace_storage::repo::security::SecurityRepo;

use crate::state::AppState;

#[derive(Deserialize)]
pub struct RecentQ {
    #[serde(default = "default_limit")]
    pub limit: i64,
}
fn default_limit() -> i64 {
    50
}

pub async fn recent(State(state): State<AppState>, Query(q): Query<RecentQ>) -> Json<Value> {
    let findings = SecurityRepo::new(&state.db)
        .recent_findings(q.limit.clamp(1, 200))
        .await
        .unwrap_or_default();
    Json(json!({ "findings": findings }))
}

#[derive(Deserialize)]
pub struct WindowQ {
    #[serde(default = "default_days")]
    pub days: i64,
}
fn default_days() -> i64 {
    30
}

pub async fn scoreboard(State(state): State<AppState>, Query(q): Query<WindowQ>) -> Json<Value> {
    let days = q.days.clamp(1, 365);
    let repo = SecurityRepo::new(&state.db);
    let counts = repo.severity_counts(days).await.unwrap_or_default();
    let counts: Vec<_> = counts
        .into_iter()
        .map(|(s, c)| json!({ "severity": severity_str(s), "count": c }))
        .collect();
    let rules = repo.rule_rankings(days, 20).await.unwrap_or_default();
    Json(json!({
        "days": days,
        "severity_counts": counts,
        "rule_rankings": rules,
    }))
}

fn severity_str(s: trace_common::model::Severity) -> &'static str {
    use trace_common::model::Severity::*;
    match s {
        Info => "info",
        Low => "low",
        Medium => "medium",
        High => "high",
        Critical => "critical",
    }
}
