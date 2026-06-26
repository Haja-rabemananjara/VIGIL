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

pub async fn update_incident_status(
    pool: &PgPool,
    incident_id: Uuid,
    new_status: &str,
    new_severity: Option<&str>,
) -> Result<IncidentRow, AppError> {
    let row = sqlx::query_as!(
        IncidentRow,
        r#"
        UPDATE incidents
        SET
            status          = $2,
            severity        = COALESCE($3, severity),
            acknowledged_at = CASE WHEN $2 = 'acknowledged' THEN now() ELSE acknowledged_at END,
            escalated_at    = CASE WHEN $2 = 'escalated'    THEN now() ELSE escalated_at    END,
            resolved_at     = CASE WHEN $2 = 'resolved'     THEN now() ELSE resolved_at     END,
            updated_at      = now()
        WHERE id = $1
        RETURNING
            id, team_id, title, body, severity, status, created_by,
            created_at, updated_at, acknowledged_at, escalated_at, resolved_at
        "#,
        incident_id,
        new_status,
        new_severity,
    )
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn update_incident_severity(
    pool: &PgPool,
    incident_id: Uuid,
    new_severity: &str,
) -> Result<IncidentRow, AppError> {
    let row = sqlx::query_as!(
        IncidentRow,
        r#"
        UPDATE incidents
        SET severity   = $2,
            updated_at = now()
        WHERE id = $1
        RETURNING
            id, team_id, title, body, severity, status, created_by,
            created_at, updated_at, acknowledged_at, escalated_at, resolved_at
        "#,
        incident_id,
        new_severity,
    )
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn insert_system_timeline_entry(
    pool: &PgPool,
    id: Uuid,
    incident_id: Uuid,
    author_id: Uuid, // the user who triggered the state change
    content: &str,
) -> Result<(), AppError> {
    sqlx::query!(
        r#"
        INSERT INTO timeline_entries (id, incident_id, author_id, kind, content, created_at)
        VALUES ($1, $2, $3, 'system', $4, now())
        "#,
        id,
        incident_id,
        author_id,
        content,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn deactivate_current_assignee(
    pool: &PgPool,
    incident_id: Uuid,
) -> Result<bool, AppError> {
    let result = sqlx::query!(
        r#"
        UPDATE incident_assignments
        SET status        = 'replaced',
            unassigned_at = now()
        WHERE incident_id = $1
          AND status      = 'active'
        "#,
        incident_id,
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn insert_assignment(
    pool: &PgPool,
    id: Uuid,
    incident_id: Uuid,
    user_id: Uuid,
    assigned_by: Uuid,
) -> Result<(), AppError> {
    sqlx::query!(
        r#"
        INSERT INTO incident_assignments
            (id, incident_id, user_id, assigned_by, status, assigned_at)
        VALUES ($1, $2, $3, $4, 'active', now())
        "#,
        id,
        incident_id,
        user_id,
        assigned_by,
    )
    .execute(pool)
    .await?;

    Ok(())
}
