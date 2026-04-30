use axum::{
    Json,
    extract::{Path, Query, State},
};
use chrono::{Duration, Utc};
use serde::Deserialize;
use serde_json::{Value, json};
use trace_storage::repo::alerts::AlertRepo;
use trace_storage::repo::analytics::AnalyticsRepo;
use trace_storage::repo::tvl::TvlRepo;

use crate::state::AppState;

#[derive(Deserialize)]
pub struct DeployQ {
    #[serde(default = "default_days")]
    pub days: i64,
}
fn default_days() -> i64 {
    30
}

pub async fn deployments(State(state): State<AppState>, Query(q): Query<DeployQ>) -> Json<Value> {
    let to = Utc::now();
    let from = to - Duration::days(q.days.clamp(1, 365));
    let stats = AnalyticsRepo::new(&state.db)
        .daily_deploys(from, to)
        .await
        .unwrap_or_default();
    let result: Vec<_> = stats
        .into_iter()
        .map(|s| {
            json!({
                "day": s.day,
                "package_count": s.package_count,
                "unique_publishers": s.unique_publishers,
            })
        })
        .collect();
    Json(json!({ "from": from, "to": to, "stats": result }))
}

#[derive(Deserialize)]
pub struct ActiveQ {
    #[serde(default = "default_window")]
    pub hours: i64,
    #[serde(default = "default_active_limit")]
    pub limit: i64,
}
fn default_window() -> i64 {
    24
}
fn default_active_limit() -> i64 {
    50
}

pub async fn active(State(state): State<AppState>, Query(q): Query<ActiveQ>) -> Json<Value> {
    let since = Utc::now() - Duration::hours(q.hours.clamp(1, 168));
    let rows = AnalyticsRepo::new(&state.db)
        .active_packages(since, q.limit.min(200))
        .await
        .unwrap_or_default();
    let result: Vec<_> = rows
        .into_iter()
        .map(|r| {
            json!({
                "package_id": r.package_id,
                "calls": r.calls,
                "unique_callers": r.unique_callers,
                "gas_total": r.gas_total,
            })
        })
        .collect();
    Json(json!({ "since": since, "rankings": result }))
}

#[derive(Deserialize)]
pub struct TvlQ {
    #[serde(default = "default_tvl_window")]
    pub hours: i64,
}
fn default_tvl_window() -> i64 {
    24
}

pub async fn tvl(
    State(state): State<AppState>,
    Path(protocol): Path<String>,
    Query(q): Query<TvlQ>,
) -> Json<Value> {
    let to = Utc::now();
    let from = to - Duration::hours(q.hours.clamp(1, 24 * 90));
    let points = TvlRepo::new(&state.db)
        .history(&protocol, from, to)
        .await
        .unwrap_or_default();
    Json(json!({ "protocol": protocol, "history": points }))
}

#[derive(Deserialize)]
pub struct ThroughputQ {
    #[serde(default = "default_minutes")]
    pub minutes: i64,
}
fn default_minutes() -> i64 {
    60
}

pub async fn throughput(
    State(state): State<AppState>,
    Query(q): Query<ThroughputQ>,
) -> Json<Value> {
    let minutes = q.minutes.clamp(5, 24 * 60);
    let points = AnalyticsRepo::new(&state.db)
        .tx_throughput(minutes)
        .await
        .unwrap_or_default();
    Json(json!({ "minutes": minutes, "points": points }))
}

#[derive(Deserialize)]
pub struct FeedQ {
    #[serde(default = "default_feed_limit")]
    pub limit: i64,
}
fn default_feed_limit() -> i64 {
    50
}

pub async fn alerts_feed(State(state): State<AppState>, Query(q): Query<FeedQ>) -> Json<Value> {
    let alerts = AlertRepo::new(&state.db)
        .recent_feed(q.limit.clamp(1, 200))
        .await
        .unwrap_or_default();
    let result: Vec<_> = alerts
        .into_iter()
        .map(|r| {
            json!({
                "id": r.id,
                "rule_id": r.rule_id,
                "fired_at": r.fired_at,
                "payload": r.payload,
                "delivered": r.delivered,
            })
        })
        .collect();
    Json(json!({ "alerts": result }))
}
