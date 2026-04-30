use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use chrono::Utc;
use serde::Deserialize;
use serde_json::{Value, json};
use trace_storage::repo::alerts::AlertRepo;
use trace_storage::repo::watchlists::{Watchlist, WatchlistRepo};
use uuid::Uuid;

use crate::auth::{AuthUser, parse_user_id};
use crate::state::AppState;

pub async fn list(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
) -> Result<Json<Value>, StatusCode> {
    let user_id = parse_user_id(&claims).ok_or(StatusCode::UNAUTHORIZED)?;
    let rows = WatchlistRepo::new(&state.db)
        .list_for_user(user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(
        json!({ "watchlists": rows.into_iter().map(serialize_wl).collect::<Vec<_>>() }),
    ))
}

#[derive(Deserialize)]
pub struct CreateBody {
    pub name: String,
    pub target_type: String,
    pub target_id: String,
    #[serde(default)]
    pub rules: Value,
    #[serde(default)]
    pub channels: Value,
}

pub async fn create(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Json(body): Json<CreateBody>,
) -> Result<Json<Value>, StatusCode> {
    let user_id = parse_user_id(&claims).ok_or(StatusCode::UNAUTHORIZED)?;
    let w = Watchlist {
        id: Uuid::new_v4(),
        user_id,
        name: body.name,
        target_type: body.target_type,
        target_id: body.target_id,
        rules: body.rules,
        channels: body.channels,
        created_at: Utc::now(),
    };
    WatchlistRepo::new(&state.db)
        .create(&w)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(json!({ "watchlist": serialize_wl(w) })))
}

pub async fn delete(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let user_id = parse_user_id(&claims).ok_or(StatusCode::UNAUTHORIZED)?;
    WatchlistRepo::new(&state.db)
        .delete(id, user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct AlertsQ {
    #[serde(default = "default_limit")]
    pub limit: i64,
}
fn default_limit() -> i64 {
    50
}

pub async fn alerts(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Query(q): Query<AlertsQ>,
) -> Result<Json<Value>, StatusCode> {
    let user_id = parse_user_id(&claims).ok_or(StatusCode::UNAUTHORIZED)?;
    let rows = AlertRepo::new(&state.db)
        .recent_for_user(user_id, q.limit.min(200))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let result: Vec<_> = rows
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
    Ok(Json(json!({ "alerts": result })))
}

fn serialize_wl(w: Watchlist) -> Value {
    json!({
        "id": w.id,
        "name": w.name,
        "target_type": w.target_type,
        "target_id": w.target_id,
        "rules": w.rules,
        "channels": w.channels,
        "created_at": w.created_at,
    })
}
