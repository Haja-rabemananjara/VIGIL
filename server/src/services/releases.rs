use std::collections::HashMap;

use uuid::Uuid;

use crate::domain::releases::{ReleaseListItem, ReleaseResponse, ReleaseStepRow};
use crate::error::AppError;
use crate::repo;
use crate::state::AppState;

pub async fn create_release(
    state: &AppState,
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
        &state.pool,
        release_id,
        team_id,
        &title,
        &body,
        created_by,
        &step_names,
    )
    .await
    .map_err(|e| AppError::Internal(format!("Failed to create release: {e}")))?;

    let steps = repo::releases::get_steps_for_release(&state.pool, release_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch steps: {e}")))?;

    Ok(ReleaseResponse::from_row(row, steps))
}

pub async fn list_releases(
    state: &AppState,
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

    let releases = repo::releases::list_releases(&state.pool, team_id, status_filter.as_deref())
        .await
        .map_err(|e| AppError::Internal(format!("Failed to list releases: {e}")))?;

    if releases.is_empty() {
        return Ok(vec![]);
    }

    let release_ids: Vec<Uuid> = releases.iter().map(|r| r.id).collect();
    let all_steps = repo::releases::get_steps_for_releases(&state.pool, &release_ids)
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
    state: &AppState,
    release_id: Uuid,
    team_id: Uuid,
) -> Result<ReleaseResponse, AppError> {
    let row = repo::releases::get_release_by_id(&state.pool, release_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch release: {e}")))?
        .ok_or_else(|| AppError::NotFound("Release not found".into()))?;

    if row.team_id != team_id {
        return Err(AppError::NotFound("Release not found".into()));
    }

    let steps = repo::releases::get_steps_for_release(&state.pool, release_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch steps: {e}")))?;

    Ok(ReleaseResponse::from_row(row, steps))
}
