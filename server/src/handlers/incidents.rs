use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::extractors::RequireResponder;
use crate::{
    error::AppError,
    extractors::{RequireManager, TeamMember},
    services,
    state::AppState,
};

#[derive(Deserialize)]
pub struct CreateIncidentBody {
    pub title: String,
    #[serde(default)]
    pub body: String,
    pub severity: String,
}

pub async fn create_incident(
    State(state): State<AppState>,
    manager: RequireManager,
    Json(body): Json<CreateIncidentBody>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let incident = services::incidents::create_incident(
        &state.pool,
        state.broadcaster,
        manager.0.team_id,
        manager.0.user_id,
        services::incidents::CreateIncidentInput {
            title: body.title,
            body: body.body,
            severity: body.severity,
        },
    )
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(incident).unwrap()),
    ))
}

#[derive(Deserialize)]
pub struct ListIncidentsQuery {
    pub status: Option<String>,
    pub severity: Option<String>,
}

pub async fn list_incidents(
    State(state): State<AppState>,
    member: TeamMember,
    Query(params): Query<ListIncidentsQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let incidents = services::incidents::list_incidents(
        &state.pool,
        services::incidents::ListIncidentsInput {
            team_id: member.team_id,
            status_filter: params.status,
            severity_filter: params.severity,
        },
    )
    .await?;

    Ok(Json(serde_json::json!({ "incidents": incidents })))
}

pub async fn get_incident(
    State(state): State<AppState>,
    member: TeamMember,
    Path((_team_id, incident_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    let incident =
        services::incidents::get_incident(&state.pool, incident_id, member.team_id).await?;

    Ok(Json(serde_json::to_value(incident).unwrap()))
}

#[derive(Deserialize)]
pub struct TransitionStatusBody {
    pub status: String,
    pub severity: Option<String>,
}

pub async fn transition_incident_status(
    State(state): State<AppState>,
    responder: RequireResponder,
    Path((_team_id, incident_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<TransitionStatusBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let incident = services::incidents::transition_incident_status(
        &state.pool,
        state.broadcaster,
        incident_id,
        responder.0.team_id,
        responder.0.user_id,
        body.status,
        body.severity,
    )
    .await?;

    Ok(Json(serde_json::to_value(incident).unwrap()))
}

#[derive(Deserialize)]
pub struct UpdateSeverityBody {
    pub severity: String,
}

pub async fn update_incident_severity(
    State(state): State<AppState>,
    responder: RequireResponder,
    Path((_team_id, incident_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateSeverityBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let incident = services::incidents::update_incident_severity(
        &state.pool,
        incident_id,
        responder.0.team_id,
        body.severity,
    )
    .await?;

    Ok(Json(serde_json::to_value(incident).unwrap()))
}

#[derive(Deserialize)]
pub struct AssignBody {
    pub user_id: Uuid,
}

pub async fn assign_responder(
    State(state): State<AppState>,
    manager: RequireManager,
    Path((_team_id, incident_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<AssignBody>,
) -> Result<StatusCode, AppError> {
    services::incidents::assign_responder(
        &state.pool,
        state.broadcaster,
        incident_id,
        manager.0.team_id,
        manager.0.user_id,
        body.user_id,
    )
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct AddTimelineEntryBody {
    pub content: String,
}

pub async fn add_timeline_entry(
    State(state): State<AppState>,
    responder: RequireResponder,
    Path((_team_id, incident_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<AddTimelineEntryBody>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let entry = services::incidents::add_timeline_entry(
        &state.pool,
        state.broadcaster,
        incident_id,
        responder.0.team_id,
        responder.0.user_id,
        body.content,
    )
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(entry).unwrap()),
    ))
}

#[derive(Deserialize)]
pub struct TimelineQuery {
    pub before: Option<i64>,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    50
}

pub async fn get_timeline(
    State(state): State<AppState>,
    member: TeamMember,
    Path((_team_id, incident_id)): Path<(Uuid, Uuid)>,
    Query(params): Query<TimelineQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let entries = services::incidents::get_timeline(
        &state.pool,
        incident_id,
        member.team_id,
        params.before,
        params.limit,
    )
    .await?;

    Ok(Json(serde_json::json!({ "entries": entries })))
}
