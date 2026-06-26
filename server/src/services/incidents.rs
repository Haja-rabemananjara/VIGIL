use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    domain::incidents::{IncidentSeverity, IncidentStatus},
    error::AppError,
    repo,
};

#[derive(Debug, Serialize)]
pub struct IncidentResponse {
    pub id: Uuid,
    pub team_id: Uuid,
    pub title: String,
    pub body: String,
    pub severity: String,
    pub status: String,
    pub created_by: Uuid,
    pub created_at: i64,
    pub updated_at: i64,
    pub acknowledged_at: Option<i64>,
    pub escalated_at: Option<i64>,
    pub resolved_at: Option<i64>,
}

impl IncidentResponse {
    fn from_row(row: crate::repo::incidents::IncidentRow) -> Self {
        Self {
            id: row.id,
            team_id: row.team_id,
            title: row.title,
            body: row.body,
            severity: row.severity,
            status: row.status,
            created_by: row.created_by,
            created_at: row.created_at.timestamp(),
            updated_at: row.updated_at.timestamp(),
            acknowledged_at: row.acknowledged_at.map(|t| t.timestamp()),
            escalated_at: row.escalated_at.map(|t| t.timestamp()),
            resolved_at: row.resolved_at.map(|t| t.timestamp()),
        }
    }
}

pub struct CreateIncidentInput {
    pub title: String,
    pub body: String,
    pub severity: String,
}

pub struct ListIncidentsInput {
    pub team_id: Uuid,
    pub status_filter: Option<String>,
    pub severity_filter: Option<String>,
}

pub async fn create_incident(
    pool: &PgPool,
    team_id: Uuid,
    created_by: Uuid,
    input: CreateIncidentInput,
) -> Result<IncidentResponse, AppError> {
    IncidentSeverity::try_from(input.severity.as_str())
        .map_err(|_| AppError::Validation(format!("unknown severity: {}", input.severity)))?;

    let title = input.title.trim().to_string();
    if title.is_empty() {
        return Err(AppError::Validation("title cannot be empty".into()));
    }

    let id = Uuid::new_v4();

    let row = repo::incidents::create_incident(
        pool,
        id,
        team_id,
        &title,
        &input.body,
        &input.severity,
        created_by,
    )
    .await?;

    Ok(IncidentResponse::from_row(row))
}

pub async fn list_incidents(
    pool: &PgPool,
    input: ListIncidentsInput,
) -> Result<Vec<IncidentResponse>, AppError> {
    if let Some(ref s) = input.status_filter {
        IncidentStatus::try_from(s.as_str())
            .map_err(|_| AppError::Validation(format!("unknown status filter: {s}")))?;
    }
    if let Some(ref s) = input.severity_filter {
        IncidentSeverity::try_from(s.as_str())
            .map_err(|_| AppError::Validation(format!("unknown severity filter: {s}")))?;
    }

    let rows = repo::incidents::list_incidents(
        pool,
        input.team_id,
        input.status_filter.as_deref(),
        input.severity_filter.as_deref(),
    )
    .await?;

    Ok(rows.into_iter().map(IncidentResponse::from_row).collect())
}

pub async fn get_incident(
    pool: &PgPool,
    incident_id: Uuid,
    team_id: Uuid,
) -> Result<IncidentResponse, AppError> {
    let row = repo::incidents::find_incident(pool, incident_id)
        .await?
        .ok_or_else(|| AppError::NotFound("incident not found".into()))?;

    if row.team_id != team_id {
        return Err(AppError::NotFound("incident not found".into()));
    }

    Ok(IncidentResponse::from_row(row))
}
