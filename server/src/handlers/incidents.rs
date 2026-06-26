use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use uuid::Uuid;

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
