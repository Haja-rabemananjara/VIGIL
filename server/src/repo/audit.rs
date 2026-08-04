use serde_json::Value as JsonValue;
use sqlx::PgPool;
use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow, serde::Serialize)]
pub struct AuditEntry {
    pub id: Uuid,
    pub team_id: Option<Uuid>,
    pub actor_id: Option<Uuid>,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub metadata: JsonValue,
    pub created_at: DateTime<Utc>,
}

pub async fn record(
    pool: &PgPool,
    team_id: Option<Uuid>,
    actor_id: Option<Uuid>,
    action: &str,
    entity_type: &str,
    entity_id: Uuid,
    metadata: JsonValue,
) -> Result<(), sqlx::Error> {
    let id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO audit_log (id, team_id, actor_id, action, entity_type, entity_id, metadata)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
        id,
        team_id,
        actor_id,
        action,
        entity_type,
        entity_id,
        metadata,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_for_team(
    pool: &PgPool,
    team_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<AuditEntry>, sqlx::Error> {
    sqlx::query_as!(
        AuditEntry,
        r#"
        SELECT id, team_id, actor_id, action, entity_type, entity_id, metadata, created_at
        FROM audit_log
        WHERE team_id = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        team_id,
        limit,
        offset,
    )
    .fetch_all(pool)
    .await
}
