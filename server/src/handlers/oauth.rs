use axum::Json;
use axum::extract::{Query, State};
use axum::response::Redirect;
use rand::RngCore;
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::handlers::auth::UserResponse;
use crate::services;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct CallbackQuery {
    pub code: String,
    pub state: String,
}

#[derive(Debug, Serialize)]
pub struct OAuthSigninResponse {
    pub token: String,
    pub user: UserResponse,
}

pub async fn github_redirect(State(state): State<AppState>) -> Result<Redirect, AppError> {
    let client_id = state
        .github_client_id
        .as_ref()
        .ok_or_else(|| AppError::Internal("GitHub OAuth not configured".into()))?;

    let csrf_state = hex::encode({
        let mut buf = [0u8; 16];
        rand::rng().fill_bytes(&mut buf);
        buf
    });

    let url = services::oauth::github_authorize_url(client_id, &csrf_state);

    Ok(Redirect::temporary(&url))
}

pub async fn github_callback(
    State(state): State<AppState>,
    Query(query): Query<CallbackQuery>,
) -> Result<Json<OAuthSigninResponse>, AppError> {
    let client_id = state
        .github_client_id
        .as_ref()
        .ok_or_else(|| AppError::Internal("GitHub OAuth not configured".into()))?;
    let client_secret = state
        .github_client_secret
        .as_ref()
        .ok_or_else(|| AppError::Internal("GitHub OAuth not configured".into()))?;

    let (token, user) = services::oauth::github_callback(
        &state.pool,
        &state.http_client,
        client_id,
        client_secret,
        &query.code,
    )
    .await?;

    Ok(Json(OAuthSigninResponse {
        token,
        user: user.into(),
    }))
}
