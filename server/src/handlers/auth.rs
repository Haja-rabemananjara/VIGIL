use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AppState;
use crate::error::AppError;
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
