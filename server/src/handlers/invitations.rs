use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;

use crate::error::AppError;
use crate::extractors::RequireManager;
use crate::handlers::auth::AuthUser;
use crate::services;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct JoinRequest {
    pub code: String,
}

pub async fn create_invitation(
    State(state): State<AppState>,
    manager: RequireManager,
) -> Result<(StatusCode, Json<services::invitations::InvitationView>), AppError> {
    let invitation =
        services::invitations::create_invitation(&state.pool, manager.0.team_id, manager.0.user_id)
            .await?;

    Ok((StatusCode::CREATED, Json(invitation)))
}

pub async fn join_team(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<JoinRequest>,
) -> Result<(StatusCode, Json<services::invitations::JoinResult>), AppError> {
    let result = services::invitations::join_team(
        &state.pool,
        state.broadcaster.clone(),
        user.id,
        &body.code,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(result)))
}
