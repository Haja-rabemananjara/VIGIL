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
#[derive(Debug, sqlx::FromRow, serde::Serialize)]
pub struct AuditEntryWithActor {
    pub id: Uuid,
    pub team_id: Option<Uuid>,
    pub actor_id: Option<Uuid>,
    pub actor_name: Option<String>,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub metadata: JsonValue,
    pub created_at: DateTime<Utc>,
}

pub async fn list_for_team(
    pool: &PgPool,
    team_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<AuditEntryWithActor>, sqlx::Error> {
    sqlx::query_as!(
        AuditEntryWithActor,
        r#"
        SELECT a.id, a.team_id, a.actor_id,
               u.display_name AS actor_name,
               a.action, a.entity_type, a.entity_id, a.metadata, a.created_at
        FROM audit_log a
        LEFT JOIN users u ON u.id = a.actor_id
        WHERE a.team_id = $1
        ORDER BY a.created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        team_id,
        limit,
        offset,
    )
    .fetch_all(pool)
    .await
}
