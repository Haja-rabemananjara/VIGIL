use crate::{
    domain,
    domain::incidents::{IncidentSeverity, IncidentStatus},
    error::AppError,
    repo,
    ws::{Broadcaster, WsEvent},
};
use domain::team;
use serde::Serialize;
use sqlx::PgPool;
use team::Role;
use uuid::Uuid;

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
    pub assignee_id: Option<Uuid>,
}

impl IncidentResponse {
    fn from_row(row: repo::incidents::IncidentRow) -> Self {
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
            assignee_id: row.assignee_id,
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
    broadcaster: Broadcaster,
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

    broadcaster
        .to_team(
            team_id,
            WsEvent::IncidentStateChanged {
                team_id,
                incident_id: id,
                new_state: "open".to_string(),
                by: created_by,
            },
        )
        .await;

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

pub async fn transition_incident_status(
    pool: &PgPool,
    broadcaster: Broadcaster,
    incident_id: Uuid,
    team_id: Uuid,
    actor_id: Uuid,
    new_status: String,
    new_severity: Option<String>,
) -> Result<IncidentResponse, AppError> {
    let to = IncidentStatus::try_from(new_status.as_str())
        .map_err(|_| AppError::Validation(format!("unknown status: {new_status}")))?;

    let row = repo::incidents::find_incident(pool, incident_id)
        .await?
        .ok_or_else(|| AppError::NotFound("incident not found".into()))?;

    if row.team_id != team_id {
        return Err(AppError::NotFound("incident not found".into()));
    }

    let from = IncidentStatus::try_from(row.status.as_str())
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if !domain::incidents::can_transition(&from, &to) {
        return Err(AppError::Validation(format!(
            "cannot transition from '{from}' to '{to}'"
        )));
    }

    let severity_to_set = match &new_severity {
        Some(s) => {
            IncidentSeverity::try_from(s.as_str())
                .map_err(|_| AppError::Validation(format!("unknown severity: {s}")))?;
            Some(s.as_str())
        }
        None => None,
    };

    let updated =
        repo::incidents::update_incident_status(pool, incident_id, &new_status, severity_to_set)
            .await?;

    let entry_content = match &to {
        IncidentStatus::Acknowledged => "Incident acknowledged".to_string(),
        IncidentStatus::Escalated => {
            if let Some(ref sev) = new_severity {
                format!("Incident escalated — severity raised to {sev}")
            } else {
                "Incident escalated".to_string()
            }
        }
        IncidentStatus::Resolved => "Incident resolved".to_string(),
        IncidentStatus::Open => "Incident reopened".to_string(),
    };

    repo::incidents::insert_system_timeline_entry(
        pool,
        Uuid::new_v4(),
        incident_id,
        actor_id,
        &entry_content,
    )
    .await?;

    broadcaster
        .to_team(
            team_id,
            WsEvent::IncidentStateChanged {
                team_id,
                incident_id,
                new_state: new_status.clone(),
                by: actor_id,
            },
        )
        .await;

    if to == IncidentStatus::Escalated
        && let Some(ref sev) = new_severity
    {
        broadcaster
            .to_team(
                team_id,
                WsEvent::IncidentEscalated {
                    team_id,
                    incident_id,
                    new_severity: sev.clone(),
                    by: actor_id,
                },
            )
            .await;
    }

    if to == IncidentStatus::Resolved {
        crate::services::releases::check_and_unblock_releases_for_incident(
            pool,
            broadcaster.clone(),
            incident_id,
        )
        .await?;
    }

    Ok(IncidentResponse::from_row(updated))
}

pub async fn update_incident_severity(
    pool: &PgPool,
    incident_id: Uuid,
    team_id: Uuid,
    new_severity: String,
) -> Result<IncidentResponse, AppError> {
    IncidentSeverity::try_from(new_severity.as_str())
        .map_err(|_| AppError::Validation(format!("unknown severity: {new_severity}")))?;

    let row = repo::incidents::find_incident(pool, incident_id)
        .await?
        .ok_or_else(|| AppError::NotFound("incident not found".into()))?;

    if row.team_id != team_id {
        return Err(AppError::NotFound("incident not found".into()));
    }

    let updated =
        repo::incidents::update_incident_severity(pool, incident_id, &new_severity).await?;

    Ok(IncidentResponse::from_row(updated))
}

pub async fn assign_responder(
    pool: &PgPool,
    broadcaster: Broadcaster,
    incident_id: Uuid,
    team_id: Uuid,
    assigned_by: Uuid,
    target_user_id: Uuid,
) -> Result<(), AppError> {
    let row = repo::incidents::find_incident(pool, incident_id)
        .await?
        .ok_or_else(|| AppError::NotFound("incident not found".into()))?;

    if row.team_id != team_id {
        return Err(AppError::NotFound("incident not found".into()));
    }

    let membership = repo::teams::find_membership(pool, team_id, target_user_id)
        .await?
        .ok_or_else(|| AppError::Validation("target user is not a member of this team".into()))?;

    let target_role = Role::from_db(membership.role.as_str())
        .ok_or_else(|| AppError::Internal("invalid role in database".into()))?;

    if !target_role.has_at_least(Role::Responder) {
        return Err(AppError::Validation(
            "target user must be at least a Responder".into(),
        ));
    }

    repo::incidents::deactivate_current_assignee(pool, incident_id).await?;

    repo::incidents::insert_assignment(
        pool,
        Uuid::new_v4(),
        incident_id,
        target_user_id,
        assigned_by,
    )
    .await?;

    broadcaster
        .to_team(
            team_id,
            WsEvent::IncidentAssigned {
                team_id,
                incident_id,
                assigned_to: target_user_id,
                by: assigned_by,
            },
        )
        .await;

    broadcaster.to_user(
        target_user_id,
        WsEvent::IncidentAssigned {
            team_id,
            incident_id,
            assigned_to: target_user_id,
            by: assigned_by,
        },
    );

    Ok(())
}

pub const TIMELINE_MAX_LENGTH: usize = 2000;

#[derive(Debug, Serialize)]
pub struct TimelineEntryResponse {
    pub id: Uuid,
    pub incident_id: Uuid,
    pub author_id: Uuid,
    pub kind: String,
    pub content: String,
    pub created_at: i64,
    pub edited_at: Option<i64>,
}

impl TimelineEntryResponse {
    fn from_row(row: repo::incidents::TimelineEntryRow) -> Self {
        Self {
            id: row.id,
            incident_id: row.incident_id,
            author_id: row.author_id,
            kind: row.kind,
            content: row.content,
            created_at: row.created_at.timestamp(),
            edited_at: row.edited_at.map(|t| t.timestamp()),
        }
    }
}

pub async fn add_timeline_entry(
    pool: &PgPool,
    broadcaster: Broadcaster,
    incident_id: Uuid,
    team_id: Uuid,
    author_id: Uuid,
    content: String,
) -> Result<TimelineEntryResponse, AppError> {
    if content.len() > TIMELINE_MAX_LENGTH {
        return Err(AppError::Validation(format!(
            "content exceeds {TIMELINE_MAX_LENGTH} characters"
        )));
    }
    if content.trim().is_empty() {
        return Err(AppError::Validation("content cannot be empty".into()));
    }

    let row = repo::incidents::find_incident(pool, incident_id)
        .await?
        .ok_or_else(|| AppError::NotFound("incident not found".into()))?;

    if row.team_id != team_id {
        return Err(AppError::NotFound("incident not found".into()));
    }

    let id = Uuid::new_v4();
    repo::incidents::insert_timeline_entry(pool, id, incident_id, author_id, &content).await?;

    let entries = repo::incidents::list_timeline_entries(pool, incident_id, None, 100).await?;
    let entry = entries
        .into_iter()
        .find(|e| e.id == id)
        .ok_or_else(|| AppError::Internal("entry not found after insert".into()))?;

    broadcaster
        .to_team(
            team_id,
            WsEvent::TimelineEntryAdded {
                team_id,
                incident_id,
                entry_id: id,
                author_id,
                content: content.clone(),
                at: entry.created_at.timestamp(),
            },
        )
        .await;

    Ok(TimelineEntryResponse::from_row(entry))
}

pub async fn get_timeline(
    pool: &PgPool,
    incident_id: Uuid,
    team_id: Uuid,
    before_ts: Option<i64>,
    limit: i64,
) -> Result<Vec<TimelineEntryResponse>, AppError> {
    let row = repo::incidents::find_incident(pool, incident_id)
        .await?
        .ok_or_else(|| AppError::NotFound("incident not found".into()))?;

    if row.team_id != team_id {
        return Err(AppError::NotFound("incident not found".into()));
    }

    let before = before_ts
        .map(|ts| chrono::DateTime::from_timestamp(ts, 0).unwrap_or_else(chrono::Utc::now));

    let limit = limit.clamp(1, 100);

    let rows = repo::incidents::list_timeline_entries(pool, incident_id, before, limit).await?;

    Ok(rows
        .into_iter()
        .map(TimelineEntryResponse::from_row)
        .collect())
}
