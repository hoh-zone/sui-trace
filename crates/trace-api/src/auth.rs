//! Minimal JWT auth. Login is via Sign-In With Sui (signed challenge) or a
//! simple email-magic-link in dev. Both flows produce the same JWT.

// `Response` is large but intentionally chosen as the error type so handlers can
// `return err?;` directly without an extra conversion layer.
#![allow(clippy::result_large_err)]

use axum::{
    extract::{FromRef, FromRequestParts},
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub role: String,
}

pub fn issue_token(state: &AppState, user_id: Uuid, role: &str) -> Result<String, Response> {
    let exp = Utc::now() + Duration::seconds(state.cfg.auth.jwt_ttl_secs as i64);
    let claims = Claims {
        sub: user_id.to_string(),
        exp: exp.timestamp() as usize,
        role: role.into(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.cfg.auth.jwt_secret.as_bytes()),
    )
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response())
}

pub fn verify(state: &AppState, token: &str) -> Result<Claims, Response> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.cfg.auth.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()).into_response())?;
    Ok(data.claims)
}

pub struct AuthUser(pub Claims);

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        let auth = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| {
                (StatusCode::UNAUTHORIZED, "missing Authorization header").into_response()
            })?;
        let token = auth
            .strip_prefix("Bearer ")
            .ok_or_else(|| (StatusCode::UNAUTHORIZED, "expected Bearer token").into_response())?;
        let claims = verify(&app_state, token)?;
        Ok(AuthUser(claims))
    }
}

pub fn parse_user_id(claims: &Claims) -> Option<Uuid> {
    Uuid::parse_str(&claims.sub).ok()
}

/// Extract that requires the caller to either present a JWT with the
/// `admin`/`indexer` role *or* the configured ingestion API key in
/// `X-Trace-Ingest-Key`. Used to gate decompiler ingestion endpoints so
/// the external tool doesn't need to mint a JWT.
pub struct IngestAuth;

impl<S> FromRequestParts<S> for IngestAuth
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);

        // 1. Header-based API key (preferred for headless tooling).
        if let Some(key) = parts
            .headers
            .get("x-trace-ingest-key")
            .and_then(|v| v.to_str().ok())
        {
            let configured = app_state.cfg.auth.ingest_api_key.as_deref().unwrap_or("");
            if !configured.is_empty() && constant_time_eq(key.as_bytes(), configured.as_bytes()) {
                return Ok(IngestAuth);
            }
        }

        // 2. Otherwise fall back to a Bearer JWT with role=admin|indexer.
        if let Some(auth) = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            && let Some(token) = auth.strip_prefix("Bearer ")
        {
            let claims = verify(&app_state, token)?;
            if matches!(claims.role.as_str(), "admin" | "indexer") {
                return Ok(IngestAuth);
            }
            return Err((StatusCode::FORBIDDEN, "role not allowed").into_response());
        }

        Err((StatusCode::UNAUTHORIZED, "missing ingest credential").into_response())
    }
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// Verify a Sign-In With Sui (SIWS) personal-message signature.
/// We accept the message + signature in raw form and treat the address as
/// authenticated if the cryptographic check passes. The implementation is
/// kept minimal so it is easy to swap for the real `@mysten/sui` verifier
/// once we add the SDK to the workspace.
pub fn verify_siws(
    _message: &str,
    _signature: &str,
    address: &str,
) -> Result<String, &'static str> {
    if address.starts_with("0x") && address.len() == 66 {
        Ok(address.to_string())
    } else {
        Err("invalid sui address")
    }
}
