use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;

pub struct IncidentRow {
    pub id: Uuid,
    pub team_id: Uuid,
    pub title: String,
    pub body: String,
    pub severity: String,
    pub status: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub escalated_at: Option<DateTime<Utc>>,
    pub resolved_at: Option<DateTime<Utc>>,
}

pub async fn create_incident(
    pool: &PgPool,
    id: Uuid,
    team_id: Uuid,
    title: &str,
    body: &str,
    severity: &str,
    created_by: Uuid,
) -> Result<IncidentRow, AppError> {
    let row = sqlx::query_as!(
        IncidentRow,
        r#"
        INSERT INTO incidents (id, team_id, title, body, severity, status, created_by, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, 'open', $6, now(), now())
        RETURNING
            id,
            team_id,
            title,
            body,
            severity,
            status,
            created_by,
            created_at,
            updated_at,
            acknowledged_at,
            escalated_at,
            resolved_at
        "#,
        id,
        team_id,
        title,
        body,
        severity,
        created_by,
    )
        .fetch_one(pool)
        .await?;

    Ok(row)
}

pub async fn list_incidents(
    pool: &PgPool,
    team_id: Uuid,
    status_filter: Option<&str>,
    severity_filter: Option<&str>,
) -> Result<Vec<IncidentRow>, AppError> {
    let rows = sqlx::query_as!(
        IncidentRow,
        r#"
        SELECT
            id,
            team_id,
            title,
            body,
            severity,
            status,
            created_by,
            created_at,
            updated_at,
            acknowledged_at,
            escalated_at,
            resolved_at
        FROM incidents
        WHERE team_id = $1
          AND ($2::text IS NULL OR status = $2)
          AND ($3::text IS NULL OR severity = $3)
        ORDER BY created_at DESC
        "#,
        team_id,
        status_filter,
        severity_filter,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn find_incident(
    pool: &PgPool,
    incident_id: Uuid,
) -> Result<Option<IncidentRow>, AppError> {
    let row = sqlx::query_as!(
        IncidentRow,
        r#"
        SELECT
            id,
            team_id,
            title,
            body,
            severity,
            status,
            created_by,
            created_at,
            updated_at,
            acknowledged_at,
            escalated_at,
            resolved_at
        FROM incidents
        WHERE id = $1
        "#,
        incident_id,
    )
    .fetch_optional(pool)
    .await?;

    Ok(row)
}
