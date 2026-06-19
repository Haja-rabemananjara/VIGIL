use axum::extract::FromRequestParts;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use axum::{Json, extract::State, http::StatusCode};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::AppState;
use crate::domain::auth::parse_bearer_token;
use crate::error::AppError;
use crate::repo;
use crate::repo::user::User;
use crate::services;

#[derive(Debug, Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    pub display_name: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub language: String,
    pub created_at: i64,
}

impl From<User> for UserResponse {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            email: u.email,
            display_name: u.display_name,
            language: u.language,
            created_at: u.created_at.timestamp(),
        }
    }
}

pub async fn signup(
    State(state): State<AppState>,
    Json(body): Json<SignupRequest>,
) -> Result<(StatusCode, Json<UserResponse>), AppError> {
    let user = services::auth::signup(&state.pool, &body.email, &body.password, &body.display_name)
        .await?;

    Ok((StatusCode::CREATED, Json(user.into())))
}

#[derive(Debug, Deserialize)]
pub struct SigninRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct SigninResponse {
    pub token: String,
    pub user: UserResponse,
}

pub async fn signin(
    State(state): State<AppState>,
    Json(body): Json<SigninRequest>,
) -> Result<Json<SigninResponse>, AppError> {
    let (token, user) = services::auth::signin(&state.pool, &body.email, &body.password).await?;

    Ok(Json(SigninResponse {
        token,
        user: user.into(),
    }))
}

pub async fn me(user: AuthUser) -> Json<UserResponse> {
    Json(UserResponse {
        id: user.id,
        email: user.email,
        display_name: user.display_name,
        language: user.language,
        created_at: user.created_at.timestamp(),
    })
}

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub language: String,
    pub created_at: DateTime<Utc>,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(AppError::Unauthorized(
                "missing authorization header".to_string(),
            ))?;

        let token_hex = parse_bearer_token(header).ok_or(AppError::Unauthorized(
            "invalid authorization format".to_string(),
        ))?;

        let token_bytes = hex::decode(token_hex)
            .map_err(|_| AppError::Unauthorized("invalid token format".to_string()))?;
        let token_hash = Sha256::digest(&token_bytes).to_vec();

        let session = repo::session::find_by_token_hash(&state.pool, &token_hash)
            .await?
            .ok_or(AppError::Unauthorized("unknown token".to_string()))?;

        if session.expires_at < chrono::Utc::now() {
            return Err(AppError::Unauthorized("token expired".to_string()));
        }

        let user = repo::user::find_by_id(&state.pool, session.user_id)
            .await?
            .ok_or(AppError::Unauthorized("user not found".to_string()))?;

        Ok(AuthUser {
            id: user.id,
            email: user.email,
            display_name: user.display_name,
            language: user.language,
            created_at: user.created_at,
        })
    }
}
