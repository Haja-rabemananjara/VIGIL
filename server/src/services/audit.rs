use serde_json::Value as JsonValue;
use sqlx::PgPool;
use uuid::Uuid;

use crate::repo;

pub async fn record(
    pool: &PgPool,
    team_id: Uuid,
    actor_id: Uuid,
    action: &str,
    entity_type: &str,
    entity_id: Uuid,
    metadata: JsonValue,
) {
    if let Err(e) = repo::audit::record(
        pool,
        Some(team_id),
        Some(actor_id),
        action,
        entity_type,
        entity_id,
        metadata,
    )
    .await
    {
        tracing::error!(error = ?e, action, "audit record failed");
    }
}

pub async fn list_for_team(
    pool: &PgPool,
    team_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<repo::audit::AuditEntryWithActor>, crate::error::AppError> {
    let entries = repo::audit::list_for_team(pool, team_id, limit, offset).await?;
    Ok(entries)
}
