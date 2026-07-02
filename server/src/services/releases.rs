use std::collections::HashMap;

use uuid::Uuid;

use crate::domain::releases::{
    ReleaseListItem, ReleaseResponse, ReleaseStatus, ReleaseStepRow, can_transition,
    can_validate_step,
};
use crate::error::AppError;
use crate::repo;
use repo::releases::get_release_by_id;
use sqlx::PgPool;

pub async fn create_release(
    pool: &PgPool,
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

    let row = repo::releases::create_release(
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

    let steps = repo::releases::get_steps_for_release(pool, release_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch steps: {e}")))?;

    Ok(ReleaseResponse::from_row(row, steps))
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

    let releases = repo::releases::list_releases(pool, team_id, status_filter.as_deref())
        .await
        .map_err(|e| AppError::Internal(format!("Failed to list releases: {e}")))?;

    if releases.is_empty() {
        return Ok(vec![]);
    }

    let release_ids: Vec<Uuid> = releases.iter().map(|r| r.id).collect();
    let all_steps = repo::releases::get_steps_for_releases(pool, &release_ids)
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

    let steps = repo::releases::get_steps_for_release(pool, release_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch steps: {e}")))?;

    Ok(ReleaseResponse::from_row(row, steps))
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

pub async fn start_release(
    pool: &PgPool,
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

    let updated =
        repo::releases::update_release_status(pool, release_id, ReleaseStatus::InProgress.as_str())
            .await
            .map_err(|e| AppError::Internal(format!("Failed to update release: {e}")))?;

    let steps = repo::releases::get_steps_for_release(pool, release_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch steps: {e}")))?;

    Ok(ReleaseResponse::from_row(updated, steps))
}

pub async fn validate_step(
    pool: &PgPool,
    release_id: Uuid,
    step_id: Uuid,
    team_id: Uuid,
    validated_by: Uuid,
) -> Result<ReleaseResponse, AppError> {
    let row = fetch_release_for_team(pool, release_id, team_id).await?;

    let current = ReleaseStatus::from_db(&row.status)
        .ok_or_else(|| AppError::Internal("Invalid release status in DB".into()))?;

    if current != ReleaseStatus::InProgress {
        return Err(AppError::Validation(format!(
            "Cannot validate steps on a release in '{}' status",
            current.as_str()
        )));
    }

    let step = repo::releases::get_step_by_id(pool, step_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch step: {e}")))?
        .ok_or_else(|| AppError::NotFound("Step not found".into()))?;

    if step.release_id != release_id {
        return Err(AppError::NotFound("Step not found".into()));
    }

    if step.validated_by.is_some() {
        return Err(AppError::Conflict("Step already validated".into()));
    }

    let all_steps = repo::releases::get_steps_for_release(pool, release_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch steps: {e}")))?;

    can_validate_step(step.position, &all_steps).map_err(AppError::Validation)?;

    repo::releases::validate_step(pool, step_id, validated_by)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to validate step: {e}")))?;

    let unvalidated_count = all_steps
        .iter()
        .filter(|s| s.validated_by.is_none())
        .count();

    let final_row = if unvalidated_count == 1 {
        repo::releases::update_release_status(pool, release_id, ReleaseStatus::Completed.as_str())
            .await
            .map_err(|e| AppError::Internal(format!("Failed to complete release: {e}")))?
    } else {
        row
    };

    let updated_steps = repo::releases::get_steps_for_release(pool, release_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch steps: {e}")))?;

    Ok(ReleaseResponse::from_row(final_row, updated_steps))
}

pub async fn cancel_release(
    pool: &PgPool,
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
        repo::releases::update_release_status(pool, release_id, ReleaseStatus::Cancelled.as_str())
            .await
            .map_err(|e| AppError::Internal(format!("Failed to cancel release: {e}")))?;

    let steps = repo::releases::get_steps_for_release(pool, release_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch steps: {e}")))?;

    Ok(ReleaseResponse::from_row(updated, steps))
}
