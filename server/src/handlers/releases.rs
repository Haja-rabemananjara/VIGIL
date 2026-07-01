use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use serde::Deserialize;
use uuid::Uuid;

use crate::domain::releases::{ReleaseListItem, ReleaseResponse};
use crate::error::AppError;
use crate::extractors::TeamMember;
use crate::services;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct CreateReleaseRequest {
    pub title: String,
    #[serde(default)]
    pub body: String,
    pub steps: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListReleasesQuery {
    pub status: Option<String>,
}

pub async fn create(
    State(state): State<AppState>,
    member: TeamMember,
    Json(payload): Json<CreateReleaseRequest>,
) -> Result<(StatusCode, Json<ReleaseResponse>), AppError> {
    if member.role.as_str() != "manager" {
        return Err(AppError::Forbidden(
            "Only the Manager can create releases".into(),
        ));
    }

    let release = services::releases::create_release(
        &state,
        member.team_id,
        member.user_id,
        payload.title,
        payload.body,
        payload.steps,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(release)))
}

pub async fn list(
    State(state): State<AppState>,
    member: TeamMember,
    Query(query): Query<ListReleasesQuery>,
) -> Result<Json<Vec<ReleaseListItem>>, AppError> {
    let releases = services::releases::list_releases(&state, member.team_id, query.status).await?;

    Ok(Json(releases))
}

pub async fn get(
    State(state): State<AppState>,
    member: TeamMember,
    Path((_team_id, release_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ReleaseResponse>, AppError> {
    let release = services::releases::get_release(&state, release_id, member.team_id).await?;

    Ok(Json(release))
}
