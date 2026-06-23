use axum::{
    Json,
    extract::{ State },
    http::StatusCode,
};
use serde::Deserialize;

use crate::domain::team::TeamView;
use crate::error::AppError;
use crate::extractors::TeamMember;
use crate::handlers::auth::AuthUser; // ← ajuste au vrai chemin (cf. ton grep)
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
