use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;

use crate::AppState;
use crate::error::AppError;
use crate::extractors::RequireManager;
use crate::repo;
use crate::services;

#[derive(Debug, Deserialize)]
pub struct AuditQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_audit_log(
    State(state): State<AppState>,
    manager: RequireManager,
    Query(query): Query<AuditQuery>,
) -> Result<Json<Vec<repo::audit::AuditEntryWithActor>>, AppError> {
    let limit = query.limit.unwrap_or(50).min(100);
    let offset = query.offset.unwrap_or(0);

    let entries =
        services::audit::list_for_team(&state.pool, manager.0.team_id, limit, offset).await?;

    Ok(Json(entries))
}
