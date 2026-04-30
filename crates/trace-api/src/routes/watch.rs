//! Curated "watched protocols" surface.
//!
//! These endpoints back the operator dashboard:
//!
//! * `GET    /api/v1/watch/dashboard`            — counters + per-protocol cards
//! * `GET    /api/v1/watch/protocols`            — list (with filter)
//! * `GET    /api/v1/watch/protocols/{id}`       — full detail (TVL + code + activity)
//! * `POST   /api/v1/watch/protocols`            — create / upsert (admin)
//! * `PUT    /api/v1/watch/protocols/{id}`       — update (admin)
//! * `DELETE /api/v1/watch/protocols/{id}`       — remove (admin)
//! * `GET    /api/v1/watch/feed/code`            — global code-update feed
//! * `GET    /api/v1/watch/feed/activity?id=`    — protocol activity (events)
//!
//! Mutating endpoints accept either `X-Trace-Ingest-Key` or a Bearer JWT
//! whose `role` is `admin`/`indexer`, via the [`IngestAuth`] extractor.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use chrono::{Duration, Utc};
use serde::Deserialize;
use serde_json::{Value, json};
use trace_storage::repo::protocols::{ProtocolRepo, ProtocolUpsert};
use trace_storage::repo::tvl::TvlRepo;

use crate::auth::IngestAuth;
use crate::state::AppState;

// ----------------------- list / detail / dashboard ------------------------

#[derive(Deserialize, Default)]
pub struct ListQ {
    /// Only return rows with `watched = true` when set.
    #[serde(default)]
    pub watched: Option<bool>,
}

pub async fn list(State(state): State<AppState>, Query(q): Query<ListQ>) -> Json<Value> {
    let only_watched = q.watched.unwrap_or(false);
    let protocols = ProtocolRepo::new(&state.db)
        .list(only_watched)
        .await
        .unwrap_or_default();
    Json(json!({ "protocols": protocols }))
}

pub async fn get_protocol(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let repo = ProtocolRepo::new(&state.db);
    let proto = repo
        .get(&id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let Some(proto) = proto else {
        return Err(StatusCode::NOT_FOUND);
    };

    let code = repo
        .recent_events_for(&proto.id, 50)
        .await
        .unwrap_or_default();
    let activity = repo
        .recent_activity(&proto.package_ids, 50)
        .await
        .unwrap_or_default();

    let since_24h = Utc::now() - Duration::hours(24);
    let activity_24h = repo
        .activity_count_since(&proto.package_ids, since_24h)
        .await
        .unwrap_or(0);

    let tvl_latest = TvlRepo::new(&state.db)
        .latest(&proto.id)
        .await
        .unwrap_or(None);

    Ok(Json(json!({
        "protocol":     proto,
        "tvl_latest":   tvl_latest,
        "activity_24h": activity_24h,
        "code_events":  code,
        "activity":     activity,
    })))
}

pub async fn dashboard(State(state): State<AppState>) -> Json<Value> {
    let repo = ProtocolRepo::new(&state.db);
    let tvl_repo = TvlRepo::new(&state.db);
    let protocols = repo.list(true).await.unwrap_or_default();

    let since_24h = Utc::now() - Duration::hours(24);
    let event_counts = repo
        .event_counts_since(since_24h)
        .await
        .unwrap_or_default()
        .into_iter()
        .collect::<std::collections::HashMap<_, _>>();

    let mut cards = Vec::with_capacity(protocols.len());
    let mut total_tvl_usd = 0.0_f64;
    let mut total_activity_24h = 0i64;
    for p in &protocols {
        let tvl = tvl_repo.latest(&p.id).await.unwrap_or(None);
        let activity = repo
            .activity_count_since(&p.package_ids, since_24h)
            .await
            .unwrap_or(0);
        let last_code = repo
            .recent_events_for(&p.id, 1)
            .await
            .unwrap_or_default()
            .into_iter()
            .next();
        if let Some(ref t) = tvl {
            total_tvl_usd += t.tvl_usd;
        }
        total_activity_24h += activity;
        cards.push(json!({
            "protocol":      p,
            "tvl_latest":    tvl,
            "activity_24h":  activity,
            "code_events_24h": event_counts.get(&p.id).copied().unwrap_or(0),
            "last_code_event": last_code,
        }));
    }
    Json(json!({
        "totals": {
            "watched":            protocols.len(),
            "tvl_usd":            total_tvl_usd,
            "activity_24h":       total_activity_24h,
            "code_events_24h":    event_counts.values().sum::<i64>(),
        },
        "cards": cards,
    }))
}

// ----------------------- code & activity feeds ----------------------------

#[derive(Deserialize, Default)]
pub struct FeedQ {
    pub limit: Option<i64>,
    /// Optional protocol filter for the activity feed.
    pub id: Option<String>,
}

pub async fn code_feed(State(state): State<AppState>, Query(q): Query<FeedQ>) -> Json<Value> {
    let limit = q.limit.unwrap_or(100).clamp(1, 500);
    let repo = ProtocolRepo::new(&state.db);
    let events = if let Some(id) = q.id.as_deref() {
        repo.recent_events_for(id, limit).await.unwrap_or_default()
    } else {
        repo.recent_events(limit).await.unwrap_or_default()
    };
    Json(json!({ "events": events }))
}

pub async fn activity_feed(
    State(state): State<AppState>,
    Query(q): Query<FeedQ>,
) -> Result<Json<Value>, StatusCode> {
    let limit = q.limit.unwrap_or(100).clamp(1, 500);
    let id = q.id.ok_or(StatusCode::BAD_REQUEST)?;
    let repo = ProtocolRepo::new(&state.db);
    let proto = repo
        .get(&id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let Some(proto) = proto else {
        return Err(StatusCode::NOT_FOUND);
    };
    let activity = repo
        .recent_activity(&proto.package_ids, limit)
        .await
        .unwrap_or_default();
    Ok(Json(json!({ "protocol_id": id, "activity": activity })))
}

// ----------------------- write paths --------------------------------------

#[derive(Deserialize)]
pub struct UpsertBody {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub package_ids: Vec<String>,
    #[serde(default = "default_category")]
    pub category: String,
    #[serde(default)]
    pub website: Option<String>,
    #[serde(default)]
    pub defillama_slug: Option<String>,
    #[serde(default = "yes")]
    pub watched: bool,
    #[serde(default)]
    pub priority: i32,
    #[serde(default = "unknown_risk")]
    pub risk_level: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub logo_url: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub treasury_addresses: Vec<String>,
    #[serde(default)]
    pub multisig_addresses: Vec<String>,
    #[serde(default)]
    pub contact: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub added_by: Option<String>,
}

fn default_category() -> String {
    "other".into()
}
fn unknown_risk() -> String {
    "unknown".into()
}
fn yes() -> bool {
    true
}

pub async fn create(
    State(state): State<AppState>,
    _auth: IngestAuth,
    Json(body): Json<UpsertBody>,
) -> Result<Json<Value>, StatusCode> {
    if body.id.is_empty() || body.name.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    let stored = ProtocolRepo::new(&state.db)
        .upsert(&ProtocolUpsert {
            id: body.id,
            name: body.name,
            package_ids: body.package_ids,
            category: body.category,
            website: body.website,
            defillama_slug: body.defillama_slug,
            watched: body.watched,
            priority: body.priority,
            risk_level: body.risk_level,
            description: body.description,
            logo_url: body.logo_url,
            tags: body.tags,
            treasury_addresses: body.treasury_addresses,
            multisig_addresses: body.multisig_addresses,
            contact: body.contact,
            notes: body.notes,
            added_by: body.added_by,
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(json!({ "protocol": stored })))
}

#[derive(Deserialize)]
pub struct UpdateBody {
    pub name: Option<String>,
    pub package_ids: Option<Vec<String>>,
    pub category: Option<String>,
    pub website: Option<String>,
    pub defillama_slug: Option<String>,
    pub watched: Option<bool>,
    pub priority: Option<i32>,
    pub risk_level: Option<String>,
    pub description: Option<String>,
    pub logo_url: Option<String>,
    pub tags: Option<Vec<String>>,
    pub treasury_addresses: Option<Vec<String>>,
    pub multisig_addresses: Option<Vec<String>>,
    pub contact: Option<String>,
    pub notes: Option<String>,
}

pub async fn update(
    State(state): State<AppState>,
    _auth: IngestAuth,
    Path(id): Path<String>,
    Json(body): Json<UpdateBody>,
) -> Result<Json<Value>, StatusCode> {
    let repo = ProtocolRepo::new(&state.db);
    let cur = repo
        .get(&id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let Some(cur) = cur else {
        return Err(StatusCode::NOT_FOUND);
    };
    let merged = ProtocolUpsert {
        id: cur.id.clone(),
        name: body.name.unwrap_or(cur.name),
        package_ids: body.package_ids.unwrap_or(cur.package_ids),
        category: body.category.unwrap_or(cur.category),
        website: body.website.or(cur.website),
        defillama_slug: body.defillama_slug.or(cur.defillama_slug),
        watched: body.watched.unwrap_or(cur.watched),
        priority: body.priority.unwrap_or(cur.priority),
        risk_level: body.risk_level.unwrap_or(cur.risk_level),
        description: body.description.or(cur.description),
        logo_url: body.logo_url.or(cur.logo_url),
        tags: body.tags.unwrap_or(cur.tags),
        treasury_addresses: body.treasury_addresses.unwrap_or(cur.treasury_addresses),
        multisig_addresses: body.multisig_addresses.unwrap_or(cur.multisig_addresses),
        contact: body.contact.or(cur.contact),
        notes: body.notes.or(cur.notes),
        added_by: cur.added_by,
    };
    let stored = repo
        .upsert(&merged)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(json!({ "protocol": stored })))
}

pub async fn remove(
    State(state): State<AppState>,
    _auth: IngestAuth,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let removed = ProtocolRepo::new(&state.db)
        .delete(&id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(if removed {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    })
}
