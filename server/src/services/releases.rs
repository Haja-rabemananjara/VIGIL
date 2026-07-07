use std::collections::HashMap;

use uuid::Uuid;

use crate::domain::releases::{
    ReleaseListItem, ReleaseResponse, ReleaseStatus, ReleaseStepRow, can_transition,
    can_validate_step,
};
use crate::error::AppError;
use crate::repo;
use crate::ws::{Broadcaster, WsEvent};
use repo::releases;
use repo::releases::get_release_by_id;
use sqlx::PgPool;

pub async fn create_release(
    pool: &PgPool,
    broadcaster: Broadcaster,
    team_id: Uuid,
    created_by: Uuid,
    title: String,
    body: String,
    step_names: Vec<String>,
) -> Result<ReleaseResponse, AppError> {
    let title = title.trim().to_string();
    if title.is_empty() {
        return Err(AppError::Validation("Title is required".into()));
    }
    if title.len() > 200 {
        return Err(AppError::Validation(
            "Title must be 200 characters or fewer".into(),
        ));
    }

    if step_names.is_empty() {
        return Err(AppError::Validation("At least one step is required".into()));
    }
    if step_names.len() > 20 {
        return Err(AppError::Validation(
            "A release can have at most 20 steps".into(),
        ));
    }

    let step_names: Vec<String> = step_names
        .into_iter()
        .map(|s| s.trim().to_string())
        .collect();

    for name in &step_names {
        if name.is_empty() {
            return Err(AppError::Validation("Step names must not be empty".into()));
        }
        if name.len() > 100 {
            return Err(AppError::Validation(
                "Step names must be 100 characters or fewer".into(),
            ));
        }
    }

    let mut seen = std::collections::HashSet::new();
    for name in &step_names {
        if !seen.insert(name.to_lowercase()) {
            return Err(AppError::Validation(format!(
                "Duplicate step name: '{name}'"
            )));
        }
    }

    let release_id = Uuid::new_v4();

    let row = releases::create_release(
        pool,
        release_id,
        team_id,
        &title,
        &body,
        created_by,
        &step_names,
    )
    .await
    .map_err(|e| AppError::Internal(format!("Failed to create release: {e}")))?;

    releases::get_steps_for_release(pool, release_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch steps: {e}")))?;

    broadcaster
        .to_team(
            team_id,
            WsEvent::ReleaseStateChanged {
                team_id,
                release_id,
                new_state: "created".to_string(),
            },
        )
        .await;

    build_full_response(pool, release_id, row).await
}

pub async fn list_releases(
    pool: &PgPool,
    team_id: Uuid,
    status_filter: Option<String>,
) -> Result<Vec<ReleaseListItem>, AppError> {
    if let Some(ref s) = status_filter {
        let valid = [
            "created",
            "in_progress",
            "completed",
            "cancelled",
            "blocked",
        ];
        if !valid.contains(&s.as_str()) {
            return Err(AppError::Validation(format!(
                "Invalid status filter: '{s}'. Must be one of: {valid:?}"
            )));
        }
    }

    let releases = releases::list_releases(pool, team_id, status_filter.as_deref())
        .await
        .map_err(|e| AppError::Internal(format!("Failed to list releases: {e}")))?;

    if releases.is_empty() {
        return Ok(vec![]);
    }

    let release_ids: Vec<Uuid> = releases.iter().map(|r| r.id).collect();
    let all_steps = releases::get_steps_for_releases(pool, &release_ids)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch steps: {e}")))?;

    let mut steps_by_release: HashMap<Uuid, Vec<ReleaseStepRow>> = HashMap::new();
    for step in all_steps {
        steps_by_release
            .entry(step.release_id)
            .or_default()
            .push(step);
    }

    let items = releases
        .into_iter()
        .map(|r| {
            let steps = steps_by_release
                .get(&r.id)
                .map(|s| s.as_slice())
                .unwrap_or(&[]);
            ReleaseListItem::from_row(r, steps)
        })
        .collect();

    Ok(items)
}

pub async fn get_release(
    pool: &PgPool,
    release_id: Uuid,
    team_id: Uuid,
) -> Result<ReleaseResponse, AppError> {
    let row = get_release_by_id(pool, release_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch release: {e}")))?
        .ok_or_else(|| AppError::NotFound("Release not found".into()))?;

    if row.team_id != team_id {
        return Err(AppError::NotFound("Release not found".into()));
    }

    releases::get_steps_for_release(pool, release_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch steps: {e}")))?;

    build_full_response(pool, release_id, row).await
}

async fn fetch_release_for_team(
    pool: &PgPool,
    release_id: Uuid,
    team_id: Uuid,
) -> Result<crate::domain::releases::ReleaseRow, AppError> {
    let row = get_release_by_id(pool, release_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch release: {e}")))?
        .ok_or_else(|| AppError::NotFound("Release not found".into()))?;

    if row.team_id != team_id {
        return Err(AppError::NotFound("Release not found".into()));
    }

    Ok(row)
}

async fn build_full_response(
    pool: &PgPool,
    release_id: Uuid,
    row: crate::domain::releases::ReleaseRow,
) -> Result<ReleaseResponse, AppError> {
    let steps = releases::get_steps_for_release(pool, release_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch steps: {e}")))?;

    let linked_rows = releases::get_linked_incidents(pool, release_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch linked incidents: {e}")))?;

    let linked = linked_rows
        .into_iter()
        .map(|r| crate::domain::releases::LinkedIncidentInfo {
            id: r.id,
            title: r.title,
            status: r.status,
            severity: r.severity,
        })
        .collect();

    Ok(ReleaseResponse::from_row(row, steps, linked))
}

pub async fn start_release(
    pool: &PgPool,
    broadcaster: Broadcaster,
    release_id: Uuid,
    team_id: Uuid,
) -> Result<ReleaseResponse, AppError> {
    let row = fetch_release_for_team(pool, release_id, team_id).await?;

    let current = ReleaseStatus::from_db(&row.status)
        .ok_or_else(|| AppError::Internal("Invalid release status in DB".into()))?;

    if !can_transition(&current, &ReleaseStatus::InProgress) {
        return Err(AppError::Validation(format!(
            "Cannot start a release in '{}' status",
            current.as_str()
        )));
    }

    let row = releases::update_release_status(pool, release_id, ReleaseStatus::InProgress.as_str())
        .await
        .map_err(|e| AppError::Internal(format!("Failed to update release: {e}")))?;

    releases::get_steps_for_release(pool, release_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch steps: {e}")))?;

    broadcaster
        .to_team(
            team_id,
            WsEvent::ReleaseStateChanged {
                team_id,
                release_id,
                new_state: "in_progress".to_string(),
            },
        )
        .await;

    build_full_response(pool, release_id, row).await
}

pub async fn validate_step(
    pool: &PgPool,
    broadcaster: Broadcaster,
    release_id: Uuid,
    step_id: Uuid,
    team_id: Uuid,
    validated_by: Uuid,
) -> Result<ReleaseResponse, AppError> {
    let row = fetch_release_for_team(pool, release_id, team_id).await?;

    let current = ReleaseStatus::from_db(&row.status)
        .ok_or_else(|| AppError::Internal("Invalid release status in DB".into()))?;

    if current == ReleaseStatus::Blocked {
        return Err(AppError::Conflict(
            "Release is blocked by an active incident. Resolve it first".into(),
        ));
    }

    if current != ReleaseStatus::InProgress {
        return Err(AppError::Validation(format!(
            "Cannot validate steps on a release in '{}' status",
            current.as_str()
        )));
    }

    let step = releases::get_step_by_id(pool, step_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch step: {e}")))?
        .ok_or_else(|| AppError::NotFound("Step not found".into()))?;

    if step.release_id != release_id {
        return Err(AppError::NotFound("Step not found".into()));
    }

    if step.validated_by.is_some() {
        return Err(AppError::Conflict("Step already validated".into()));
    }

    let all_steps = releases::get_steps_for_release(pool, release_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch steps: {e}")))?;

    can_validate_step(step.position, &all_steps).map_err(AppError::Validation)?;

    releases::validate_step(pool, step_id, validated_by)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to validate step: {e}")))?;

    broadcaster
        .to_team(
            team_id,
            WsEvent::ReleaseStepValidated {
                team_id,
                release_id,
                step_id,
                step_name: step.name.clone(),
                by: validated_by,
            },
        )
        .await;

    let unvalidated_count = all_steps
        .iter()
        .filter(|s| s.validated_by.is_none())
        .count();

    let final_row = if unvalidated_count == 1 {
        releases::update_release_status(pool, release_id, ReleaseStatus::Completed.as_str())
            .await
            .map_err(|e| AppError::Internal(format!("Failed to complete release: {e}")))?
    } else {
        row
    };

    broadcaster
        .to_team(
            team_id,
            WsEvent::ReleaseStateChanged {
                team_id,
                release_id,
                new_state: "completed".to_string(),
            },
        )
        .await;

    releases::get_steps_for_release(pool, release_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch steps: {e}")))?;

    build_full_response(pool, release_id, final_row).await
}

pub async fn cancel_release(
    pool: &PgPool,
    broadcaster: Broadcaster,
    release_id: Uuid,
    team_id: Uuid,
) -> Result<ReleaseResponse, AppError> {
    let row = fetch_release_for_team(pool, release_id, team_id).await?;

    let current = ReleaseStatus::from_db(&row.status)
        .ok_or_else(|| AppError::Internal("Invalid release status in DB".into()))?;

    if !current.can_cancel() {
        return Err(AppError::Validation(format!(
            "Cannot cancel a release in '{}' status",
            current.as_str()
        )));
    }

    let updated =
        releases::update_release_status(pool, release_id, ReleaseStatus::Cancelled.as_str())
            .await
            .map_err(|e| AppError::Internal(format!("Failed to cancel release: {e}")))?;

    releases::get_steps_for_release(pool, release_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch steps: {e}")))?;

    broadcaster
        .to_team(
            team_id,
            WsEvent::ReleaseStateChanged {
                team_id,
                release_id,
                new_state: "cancelled".to_string(),
            },
        )
        .await;

    build_full_response(pool, release_id, updated).await
}

pub async fn link_incident(
    pool: &PgPool,
    broadcaster: Broadcaster,
    release_id: Uuid,
    incident_id: Uuid,
    team_id: Uuid,
    linked_by: Uuid,
) -> Result<ReleaseResponse, AppError> {
    let release = fetch_release_for_team(pool, release_id, team_id).await?;

    let current = ReleaseStatus::from_db(&release.status)
        .ok_or_else(|| AppError::Internal("Invalid release status".into()))?;

    if current.is_terminal() {
        return Err(AppError::Validation(format!(
            "Cannot link incidents to a '{}' release",
            current.as_str()
        )));
    }

    let incident = repo::incidents::find_incident(pool, incident_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch incident: {e}")))?
        .ok_or_else(|| AppError::NotFound("Incident not found".into()))?;

    if incident.team_id != team_id {
        return Err(AppError::NotFound("Incident not found".into()));
    }

    let already_linked = releases::has_active_link(pool, release_id, incident_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to check link: {e}")))?;

    if already_linked {
        return Err(AppError::Conflict(
            "This incident is already linked to this release".into(),
        ));
    }

    releases::create_incident_link(pool, Uuid::new_v4(), release_id, incident_id, linked_by)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to create link: {e}")))?;

    let final_row = if current == ReleaseStatus::InProgress && incident.status != "resolved" {
        let blocked =
            releases::update_release_status(pool, release_id, ReleaseStatus::Blocked.as_str())
                .await
                .map_err(|e| AppError::Internal(format!("Failed to block release: {e}")))?;

        broadcaster
            .to_team(
                team_id,
                WsEvent::ReleaseStateChanged {
                    team_id,
                    release_id,
                    new_state: "blocked".to_string(),
                },
            )
            .await;

        blocked
    } else {
        release
    };

    releases::get_steps_for_release(pool, release_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch steps: {e}")))?;

    build_full_response(pool, release_id, final_row).await
}

pub async fn unlink_incident(
    pool: &PgPool,
    broadcaster: Broadcaster,
    release_id: Uuid,
    incident_id: Uuid,
    team_id: Uuid,
) -> Result<ReleaseResponse, AppError> {
    let release = fetch_release_for_team(pool, release_id, team_id).await?;

    let removed = releases::remove_incident_link(pool, release_id, incident_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to remove link: {e}")))?;

    if !removed {
        return Err(AppError::NotFound(
            "No active link between this release and incident".into(),
        ));
    }

    let current = ReleaseStatus::from_db(&release.status)
        .ok_or_else(|| AppError::Internal("Invalid release status".into()))?;

    let final_row = if current == ReleaseStatus::Blocked {
        try_unblock_release(pool, &broadcaster, release_id, team_id).await?
    } else {
        release
    };

    releases::get_steps_for_release(pool, release_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch steps: {e}")))?;

    build_full_response(pool, release_id, final_row).await
}

async fn try_unblock_release(
    pool: &PgPool,
    broadcaster: &Broadcaster,
    release_id: Uuid,
    team_id: Uuid,
) -> Result<crate::domain::releases::ReleaseRow, AppError> {
    let count = releases::count_active_unresolved_links(pool, release_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to count links: {e}")))?;

    if count == 0 {
        let unblocked =
            releases::update_release_status(pool, release_id, ReleaseStatus::InProgress.as_str())
                .await
                .map_err(|e| AppError::Internal(format!("Failed to unblock release: {e}")))?;

        broadcaster
            .to_team(
                team_id,
                WsEvent::ReleaseStateChanged {
                    team_id,
                    release_id,
                    new_state: "in_progress".to_string(),
                },
            )
            .await;

        Ok(unblocked)
    } else {
        get_release_by_id(pool, release_id)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to fetch release: {e}")))?
            .ok_or_else(|| AppError::Internal("Release disappeared".into()))
    }
}

pub async fn check_and_unblock_releases_for_incident(
    pool: &PgPool,
    broadcaster: Broadcaster,
    incident_id: Uuid,
) -> Result<(), AppError> {
    let blocked_releases = releases::get_blocked_releases_for_incident(pool, incident_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to find linked releases: {e}")))?;

    for release in blocked_releases {
        try_unblock_release(pool, &broadcaster, release.id, release.team_id).await?;
    }

    Ok(())
}
