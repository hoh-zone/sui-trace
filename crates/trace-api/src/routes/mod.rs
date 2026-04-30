use axum::{
    Router,
    routing::{delete, get, post},
};
use tower_http::cors::{Any, CorsLayer};

use crate::state::AppState;
use crate::ws::ws_handler;

pub mod address;
pub mod auth;
pub mod checkpoint;
pub mod health;
pub mod labels;
pub mod network;
pub mod packages;
pub mod protocols;
pub mod search;
pub mod security;
pub mod source;
pub mod stats;
pub mod tx;
pub mod watch;
pub mod watchlist;

pub fn router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/health", get(health::health))
        .route("/ws", get(ws_handler))
        // Transactions
        .route("/api/v1/tx/{digest}", get(tx::get_tx))
        .route("/api/v1/tx/{digest}/full", get(tx::full))
        .route("/api/v1/tx/latest", get(tx::latest))
        // Addresses
        .route("/api/v1/address/{addr}", get(address::get_address))
        .route(
            "/api/v1/address/{addr}/transactions",
            get(address::transactions),
        )
        .route("/api/v1/address/{addr}/events", get(address::events))
        // Packages
        .route("/api/v1/package/recent", get(packages::recent))
        .route("/api/v1/package/{id}", get(packages::get_package))
        .route("/api/v1/package/{id}/security", get(packages::get_security))
        .route("/api/v1/package/{id}/events", get(packages::events))
        // Versions + decompiled source
        .route("/api/v1/package/{id}/versions", get(source::versions))
        .route(
            "/api/v1/package/{id}/source",
            get(source::list_sources).post(source::ingest_source),
        )
        .route(
            "/api/v1/package/{id}/source/{module}",
            get(source::get_module_source),
        )
        // Checkpoints
        .route("/api/v1/checkpoint/recent", get(checkpoint::recent))
        .route("/api/v1/checkpoint/latest", get(checkpoint::latest))
        .route("/api/v1/checkpoint/{seq}", get(checkpoint::get_checkpoint))
        .route(
            "/api/v1/checkpoint/{seq}/transactions",
            get(checkpoint::transactions),
        )
        // Search
        .route("/api/v1/search", get(search::search))
        // Labels
        .route("/api/v1/labels/search", get(labels::search))
        .route("/api/v1/labels/{addr}", get(labels::for_address))
        .route("/api/v1/labels", post(labels::submit))
        // Stats
        .route("/api/v1/stats/deployments", get(stats::deployments))
        .route("/api/v1/stats/active", get(stats::active))
        .route("/api/v1/stats/throughput", get(stats::throughput))
        .route("/api/v1/stats/tvl/{protocol}", get(stats::tvl))
        // Network + protocols
        .route("/api/v1/network/overview", get(network::overview))
        .route("/api/v1/protocols", get(protocols::list))
        // Curated protocol watchlist (operator-facing)
        .route("/api/v1/watch/dashboard", get(watch::dashboard))
        .route(
            "/api/v1/watch/protocols",
            get(watch::list).post(watch::create),
        )
        .route(
            "/api/v1/watch/protocols/{id}",
            get(watch::get_protocol)
                .put(watch::update)
                .delete(watch::remove),
        )
        .route("/api/v1/watch/feed/code", get(watch::code_feed))
        .route("/api/v1/watch/feed/activity", get(watch::activity_feed))
        // Security
        .route("/api/v1/security/recent", get(security::recent))
        .route("/api/v1/security/scoreboard", get(security::scoreboard))
        // Auth + watchlists
        .route("/api/v1/auth/siws", post(auth::siws_login))
        .route(
            "/api/v1/watchlists",
            get(watchlist::list).post(watchlist::create),
        )
        .route("/api/v1/watchlists/{id}", delete(watchlist::delete))
        .route("/api/v1/alerts/recent", get(watchlist::alerts))
        .route("/api/v1/alerts/feed", get(stats::alerts_feed))
        .with_state(state)
        .layer(cors)
}
