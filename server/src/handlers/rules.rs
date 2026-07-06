use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use uuid::Uuid;

use crate::domain::rules::{CreateRuleInput, Rule, UpdateRuleInput};
use crate::error::AppError;
use crate::extractors::RequireManager;
use crate::services;
use crate::state::AppState;

pub async fn create(
    State(state): State<AppState>,
    RequireManager(member): RequireManager,
    Path(team_id): Path<Uuid>,
    Json(input): Json<CreateRuleInput>,
) -> Result<(StatusCode, Json<Rule>), AppError> {
    let rule = services::rules::create_rule(&state.pool, team_id, member.user_id, input).await?;
    Ok((StatusCode::CREATED, Json(rule)))
}

pub async fn list(
    State(state): State<AppState>,
    _member: crate::extractors::TeamMember,
    Path(team_id): Path<Uuid>,
) -> Result<Json<Vec<Rule>>, AppError> {
    let rules = services::rules::list_rules(&state.pool, team_id).await?;
    Ok(Json(rules))
}

pub async fn get_one(
    State(state): State<AppState>,
    _member: crate::extractors::TeamMember,
    Path((team_id, rule_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Rule>, AppError> {
    let rule = services::rules::get_rule(&state.pool, team_id, rule_id).await?;
    Ok(Json(rule))
}

pub async fn update(
    State(state): State<AppState>,
    RequireManager(_member): RequireManager,
    Path((team_id, rule_id)): Path<(Uuid, Uuid)>,
    Json(input): Json<UpdateRuleInput>,
) -> Result<Json<Rule>, AppError> {
    let rule = services::rules::update_rule(&state.pool, team_id, rule_id, input).await?;
    Ok(Json(rule))
}

pub async fn delete(
    State(state): State<AppState>,
    RequireManager(_member): RequireManager,
    Path((team_id, rule_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    services::rules::delete_rule(&state.pool, team_id, rule_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
