use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use serde_json::{Value, json};
use trace_storage::repo::users::UserRepo;

use crate::auth::{issue_token, verify_siws};
use crate::state::AppState;

#[derive(Deserialize)]
pub struct SiwsBody {
    pub address: String,
    pub message: String,
    pub signature: String,
}

pub async fn siws_login(
    State(state): State<AppState>,
    Json(body): Json<SiwsBody>,
) -> Result<Json<Value>, StatusCode> {
    let address = verify_siws(&body.message, &body.signature, &body.address)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let user = UserRepo::new(&state.db)
        .upsert_by_address(&address)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let token =
        issue_token(&state, user.id, &user.role).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(json!({ "token": token, "user": {
        "id": user.id,
        "address": user.sui_address,
        "role": user.role,
    } })))
}
