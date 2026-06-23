use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::domain::team::TeamView;
use crate::error::AppError;
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
    user: AuthUser,
    Path(team_id): Path<Uuid>,
) -> Result<Json<TeamView>, AppError> {
    let team = services::teams::get_team(&state.pool, user.id, team_id).await?;
    Ok(Json(team))
}
