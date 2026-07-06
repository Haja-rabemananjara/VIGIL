use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::rules::Rule;

pub struct NewRule<'a> {
    pub id: Uuid,
    pub team_id: Uuid,
    pub name: &'a str,
    pub enabled: bool,
    pub trigger_service: &'a str,
    pub trigger_event: &'a str,
    pub trigger_filters: &'a serde_json::Value,
    pub reaction_type: &'a str,
    pub reaction_payload: &'a serde_json::Value,
    pub created_by: Uuid,
}

pub async fn insert_rule(pool: &PgPool, rule: NewRule<'_>) -> Result<Rule, sqlx::Error> {
    let row = sqlx::query_as!(
        Rule,
        r#"
        INSERT INTO rules (
            id, team_id, name, enabled,
            trigger_service, trigger_event, trigger_filters,
            reaction_type, reaction_payload, created_by
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING
            id, team_id, name, enabled,
            trigger_service, trigger_event,
            trigger_filters as "trigger_filters: serde_json::Value",
            reaction_type,
            reaction_payload as "reaction_payload: serde_json::Value",
            created_by, created_at, updated_at
        "#,
        rule.id,
        rule.team_id,
        rule.name,
        rule.enabled,
        rule.trigger_service,
        rule.trigger_event,
        rule.trigger_filters,
        rule.reaction_type,
        rule.reaction_payload,
        rule.created_by,
    )
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn list_rules_by_team(pool: &PgPool, team_id: Uuid) -> Result<Vec<Rule>, sqlx::Error> {
    let rows = sqlx::query_as!(
        Rule,
        r#"
        SELECT
            id, team_id, name, enabled,
            trigger_service, trigger_event,
            trigger_filters as "trigger_filters: serde_json::Value",
            reaction_type,
            reaction_payload as "reaction_payload: serde_json::Value",
            created_by, created_at, updated_at
        FROM rules
        WHERE team_id = $1
        ORDER BY created_at DESC
        "#,
        team_id,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn find_rule(
    pool: &PgPool,
    team_id: Uuid,
    rule_id: Uuid,
) -> Result<Option<Rule>, sqlx::Error> {
    let row = sqlx::query_as!(
        Rule,
        r#"
        SELECT
            id, team_id, name, enabled,
            trigger_service, trigger_event,
            trigger_filters as "trigger_filters: serde_json::Value",
            reaction_type,
            reaction_payload as "reaction_payload: serde_json::Value",
            created_by, created_at, updated_at
        FROM rules
        WHERE team_id = $1 AND id = $2
        "#,
        team_id,
        rule_id,
    )
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub struct RulePatch<'a> {
    pub name: Option<&'a str>,
    pub enabled: Option<bool>,
    pub trigger_service: Option<&'a str>,
    pub trigger_event: Option<&'a str>,
    pub trigger_filters: Option<&'a serde_json::Value>,
    pub reaction_type: Option<&'a str>,
    pub reaction_payload: Option<&'a serde_json::Value>,
}

pub async fn update_rule(
    pool: &PgPool,
    team_id: Uuid,
    rule_id: Uuid,
    patch: RulePatch<'_>,
) -> Result<Option<Rule>, sqlx::Error> {
    let row = sqlx::query_as!(
        Rule,
        r#"
        UPDATE rules SET
            name = COALESCE($3, name),
            enabled = COALESCE($4, enabled),
            trigger_service = COALESCE($5, trigger_service),
            trigger_event = COALESCE($6, trigger_event),
            trigger_filters = COALESCE($7, trigger_filters),
            reaction_type = COALESCE($8, reaction_type),
            reaction_payload = COALESCE($9, reaction_payload),
            updated_at = now()
        WHERE team_id = $1 AND id = $2
        RETURNING
            id, team_id, name, enabled,
            trigger_service, trigger_event,
            trigger_filters as "trigger_filters: serde_json::Value",
            reaction_type,
            reaction_payload as "reaction_payload: serde_json::Value",
            created_by, created_at, updated_at
        "#,
        team_id,
        rule_id,
        patch.name,
        patch.enabled,
        patch.trigger_service,
        patch.trigger_event,
        patch.trigger_filters,
        patch.reaction_type,
        patch.reaction_payload,
    )
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn delete_rule(pool: &PgPool, team_id: Uuid, rule_id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"DELETE FROM rules WHERE team_id = $1 AND id = $2"#,
        team_id,
        rule_id,
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}
