use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    error::AppError, extractors::TeamMember, handlers::auth::AuthUser, services, state::AppState,
};

pub async fn get_available(_auth_user: AuthUser) -> Json<serde_json::Value> {
    let emojis = services::reactions::get_available();
    Json(serde_json::json!({ "emojis": emojis }))
}

pub async fn get_incident_reactions(
    State(state): State<AppState>,
    member: TeamMember,
    Path((_team_id, incident_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    let data =
        services::reactions::get_reactions_for_incident(&state.pool, incident_id, member.team_id)
            .await?;

    Ok(Json(serde_json::to_value(data).unwrap()))
}

#[derive(Deserialize)]
pub struct AddReactionBody {
    pub emoji: String,
}

pub async fn add_reaction(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(entry_id): Path<Uuid>,
    Json(body): Json<AddReactionBody>,
) -> Result<StatusCode, AppError> {
    services::reactions::add_reaction(
        &state.pool,
        state.broadcaster,
        entry_id,
        auth_user.id,
        body.emoji,
    )
    .await?;

    Ok(StatusCode::CREATED)
}

pub async fn remove_reaction(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((entry_id, emoji)): Path<(Uuid, String)>,
) -> Result<StatusCode, AppError> {
    services::reactions::remove_reaction(
        &state.pool,
        state.broadcaster,
        entry_id,
        auth_user.id,
        emoji,
    )
    .await?;

    Ok(StatusCode::NO_CONTENT)
}
