use axum::{Json, extract::Path, extract::State, http::StatusCode};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;

use crate::domain::team::{Role, TeamView};
use crate::error::AppError;
use crate::extractors::{RequireManager, TeamMember};
use crate::handlers::auth::AuthUser;
use crate::services;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct CreateTeamRequest {
    pub name: String,
}

pub async fn create_team(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<CreateTeamRequest>,
) -> Result<(StatusCode, Json<TeamView>), AppError> {
    let team = services::teams::create_team(&state.pool, user.id, &body.name).await?;
    Ok((StatusCode::CREATED, Json(team)))
}

pub async fn list_teams(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<TeamView>>, AppError> {
    let teams = services::teams::list_my_teams(&state.pool, user.id).await?;
    Ok(Json(teams))
}

pub async fn get_team(
    State(state): State<AppState>,
    member: TeamMember,
) -> Result<Json<TeamView>, AppError> {
    let team =
        services::teams::get_team_as_member(&state.pool, member.team_id, member.role).await?;
    Ok(Json(team))
}

pub async fn list_members(
    State(state): State<AppState>,
    member: TeamMember,
) -> Result<Json<Vec<services::teams::MemberView>>, AppError> {
    let members = services::teams::list_members(&state.pool, member.team_id).await?;
    Ok(Json(members))
}

#[derive(Deserialize)]
pub struct ChangeRoleRequest {
    pub role: String,
}

pub async fn change_member_role(
    State(state): State<AppState>,
    manager: RequireManager,
    Path((_, target_user_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<ChangeRoleRequest>,
) -> Result<StatusCode, AppError> {
    let new_role = Role::from_db(&body.role)
        .ok_or_else(|| AppError::Validation(format!("invalid role: {}", body.role)))?;

    services::teams::change_member_role(
        &state.pool,
        state.broadcaster,
        manager.0.user_id,
        manager.0.team_id,
        target_user_id,
        new_role,
    )
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct TransferManagerRequest {
    pub target_user_id: Uuid,
}

pub async fn transfer_manager(
    State(state): State<AppState>,
    manager: RequireManager,
    Json(body): Json<TransferManagerRequest>,
) -> Result<StatusCode, AppError> {
    services::teams::transfer_manager(
        &state.pool,
        state.broadcaster,
        manager.0.team_id,
        manager.0.user_id,
        body.target_user_id,
    )
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn leave_team(
    State(state): State<AppState>,
    member: TeamMember,
) -> Result<StatusCode, AppError> {
    services::teams::leave_team(&state.pool, member.team_id, member.user_id, member.role).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct BanBody {
    pub expires_at: Option<i64>,
    pub reason: Option<String>,
}

pub async fn kick_member(
    State(state): State<AppState>,
    manager: RequireManager,
    Path((_team_id, target_user_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    services::teams::kick_member(
        &state.pool,
        state.broadcaster.clone(),
        manager.0.team_id,
        manager.0.user_id,
        target_user_id,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn ban_member(
    State(state): State<AppState>,
    manager: RequireManager,
    Path((_team_id, target_user_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<BanBody>,
) -> Result<StatusCode, AppError> {
    let expires_at = body
        .expires_at
        .and_then(|ts| DateTime::<Utc>::from_timestamp(ts, 0));

    services::teams::ban_member(
        &state.pool,
        state.broadcaster.clone(),
        manager.0.team_id,
        manager.0.user_id,
        target_user_id,
        expires_at,
        body.reason,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn unban_member(
    State(state): State<AppState>,
    manager: RequireManager,
    Path((_team_id, target_user_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    services::teams::unban_member(
        &state.pool,
        manager.0.team_id,
        manager.0.user_id,
        target_user_id,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}
