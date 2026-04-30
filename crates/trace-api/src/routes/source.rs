//! Package version lineage + decompiled module source endpoints.
//!
//! `GET  /api/v1/package/{id}/versions`             — full upgrade lineage
//! `GET  /api/v1/package/{id}/source`               — list module sources (no body)
//! `GET  /api/v1/package/{id}/source/{module}`      — single module source body
//! `POST /api/v1/package/{id}/source`               — ingest from external decompiler
//!
//! The POST handler is gated by [`IngestAuth`] so the external decompiler
//! tool can authenticate using either the configured ingestion API key
//! (`X-Trace-Ingest-Key`) or an admin/indexer-role JWT.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use serde_json::{Value, json};
use trace_storage::repo::source::{ModuleSourceUpsert, SourceRepo};

use crate::auth::IngestAuth;
use crate::state::AppState;

pub async fn versions(State(state): State<AppState>, Path(id): Path<String>) -> Json<Value> {
    let lineage = SourceRepo::new(&state.db)
        .lineage_for(&id)
        .await
        .unwrap_or_default();
    Json(json!({ "package_id": id, "versions": lineage }))
}

pub async fn list_sources(State(state): State<AppState>, Path(id): Path<String>) -> Json<Value> {
    let modules = SourceRepo::new(&state.db)
        .list_modules(&id)
        .await
        .unwrap_or_default();
    Json(json!({ "package_id": id, "modules": modules }))
}

#[derive(Deserialize)]
pub struct GetSourceQ {
    /// Optional format selector: `move-source` | `pseudo` | `move-disasm`.
    /// When omitted the highest-fidelity available format is returned.
    #[serde(default)]
    pub format: Option<String>,
}

pub async fn get_module_source(
    State(state): State<AppState>,
    Path((id, module)): Path<(String, String)>,
    Query(q): Query<GetSourceQ>,
) -> Result<Json<Value>, StatusCode> {
    let src = SourceRepo::new(&state.db)
        .get_module(&id, &module, q.format.as_deref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let Some(src) = src else {
        return Err(StatusCode::NOT_FOUND);
    };
    Ok(Json(json!({ "source": src })))
}

#[derive(Deserialize)]
pub struct IngestBody {
    pub module_name: String,
    /// `move-disasm` (default), `move-source`, or `pseudo`.
    #[serde(default = "default_format")]
    pub format: String,
    pub source: String,
    /// Decompiler tool name (e.g. `sui-disassembler`, `revela`).
    #[serde(default = "default_decompiler")]
    pub decompiler: String,
    #[serde(default)]
    pub decompiler_version: Option<String>,
    /// Optional sha256 of the bytecode that produced this source (hex).
    #[serde(default)]
    pub bytecode_hash: Option<String>,
}
fn default_format() -> String {
    "move-disasm".into()
}
fn default_decompiler() -> String {
    "unknown".into()
}

pub async fn ingest_source(
    State(state): State<AppState>,
    _auth: IngestAuth,
    Path(id): Path<String>,
    Json(body): Json<IngestBody>,
) -> Result<Json<Value>, StatusCode> {
    if body.module_name.is_empty() || body.source.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    let stored = SourceRepo::new(&state.db)
        .upsert_module_source(&ModuleSourceUpsert {
            package_id: &id,
            module_name: &body.module_name,
            format: &body.format,
            source: &body.source,
            decompiler: &body.decompiler,
            decompiler_version: body.decompiler_version.as_deref(),
            bytecode_hash: body.bytecode_hash.as_deref(),
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(json!({ "source": stored })))
}
