use serde::Serialize;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::rules::{CreateRuleInput, Rule, UpdateRuleInput};
use crate::error::AppError;
use crate::hooks::{ActionCatalog, ReactionRegistry};
use crate::repo::{
    self,
    rules::{NewRule, RulePatch},
};
use crate::services::audit;
use crate::ws::{Broadcaster, WsEvent};

pub struct RuleContext<'a> {
    pub pool: &'a PgPool,
    pub broadcaster: Broadcaster,
    pub catalog: &'a ActionCatalog,
    pub registry: &'a ReactionRegistry,
}

pub async fn create_rule(
    ctx: &RuleContext<'_>,
    team_id: Uuid,
    actor_id: Uuid,
    input: CreateRuleInput,
) -> Result<Rule, AppError> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation(
            "Rule name cannot be empty".to_string(),
        ));
    }

    if !ctx.registry.contains(&input.reaction.reaction_type) {
        return Err(AppError::Validation(format!(
            "Unknown reaction: {}",
            input.reaction.reaction_type
        )));
    }

    if !ctx
        .catalog
        .contains(&input.trigger.service, &input.trigger.event)
    {
        return Err(AppError::Validation(format!(
            "Unknown trigger: {}.{}",
            input.trigger.service, input.trigger.event
        )));
    }

    let filters = if input.trigger.filters.is_null() {
        json!({})
    } else {
        input.trigger.filters
    };
    let payload = if input.reaction.payload.is_null() {
        json!({})
    } else {
        input.reaction.payload
    };

    let rule = repo::rules::insert_rule(
        ctx.pool,
        NewRule {
            id: Uuid::new_v4(),
            team_id,
            name: input.name.trim(),
            enabled: input.enabled,
            trigger_service: &input.trigger.service,
            trigger_event: &input.trigger.event,
            trigger_filters: &filters,
            reaction_type: &input.reaction.reaction_type,
            reaction_payload: &payload,
            created_by: actor_id,
        },
    )
    .await?;

    ctx.broadcaster
        .to_team(
            team_id,
            WsEvent::RuleCreated {
                team_id,
                rule_id: rule.id,
            },
        )
        .await;

    audit::record(
        ctx.pool,
        team_id,
        actor_id,
        "rule_created",
        "rule",
        rule.id,
        json!({ "name": &rule.name }),
    )
    .await;

    Ok(rule)
}

pub async fn list_rules(pool: &PgPool, team_id: Uuid) -> Result<Vec<Rule>, AppError> {
    Ok(repo::rules::list_rules_by_team(pool, team_id).await?)
}

pub async fn get_rule(pool: &PgPool, team_id: Uuid, rule_id: Uuid) -> Result<Rule, AppError> {
    repo::rules::find_rule(pool, team_id, rule_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Rule not found".to_string()))
}

pub async fn update_rule(
    ctx: &RuleContext<'_>,
    team_id: Uuid,
    rule_id: Uuid,
    actor_id: Uuid,
    input: UpdateRuleInput,
) -> Result<Rule, AppError> {
    if let Some(trigger) = &input.trigger
        && !ctx.catalog.contains(&trigger.service, &trigger.event)
    {
        return Err(AppError::Validation(format!(
            "Unknown trigger: {}.{}",
            trigger.service, trigger.event
        )));
    }

    if let Some(reaction) = &input.reaction
        && !ctx.registry.contains(&reaction.reaction_type)
    {
        return Err(AppError::Validation(format!(
            "Unknown reaction: {}",
            reaction.reaction_type
        )));
    }
    if let Some(name) = &input.name
        && name.trim().is_empty()
    {
        return Err(AppError::Validation(
            "Rule name cannot be empty".to_string(),
        ));
    }

    let trimmed_name = input.name.as_ref().map(|s| s.trim().to_string());
    let filters_value = input.trigger.as_ref().map(|t| t.filters.clone());
    let payload_value = input.reaction.as_ref().map(|r| r.payload.clone());

    let patch = RulePatch {
        name: trimmed_name.as_deref(),
        enabled: input.enabled,
        trigger_service: input.trigger.as_ref().map(|t| t.service.as_str()),
        trigger_event: input.trigger.as_ref().map(|t| t.event.as_str()),
        trigger_filters: filters_value.as_ref(),
        reaction_type: input.reaction.as_ref().map(|r| r.reaction_type.as_str()),
        reaction_payload: payload_value.as_ref(),
    };

    let rule = repo::rules::update_rule(ctx.pool, team_id, rule_id, patch)
        .await?
        .ok_or_else(|| AppError::NotFound("Rule not found".to_string()))?;

    ctx.broadcaster
        .to_team(team_id, WsEvent::RuleUpdated { team_id, rule_id })
        .await;

    audit::record(
        ctx.pool,
        team_id,
        actor_id,
        "rule_updated",
        "rule",
        rule.id,
        json!({ "name": &rule.name }),
    )
    .await;

    Ok(rule)
}

pub async fn delete_rule(
    ctx: &RuleContext<'_>,
    team_id: Uuid,
    rule_id: Uuid,
    actor_id: Uuid,
) -> Result<(), AppError> {
    let deleted = repo::rules::delete_rule(ctx.pool, team_id, rule_id).await?;
    if !deleted {
        return Err(AppError::NotFound("Rule not found".to_string()));
    }
    ctx.broadcaster
        .to_team(team_id, WsEvent::RuleDeleted { team_id, rule_id })
        .await;

    audit::record(
        ctx.pool,
        team_id,
        actor_id,
        "rule_deleted",
        "rule",
        rule_id,
        json!({}),
    )
    .await;

    Ok(())
}

#[derive(Serialize)]
pub struct ExecutionResponse {
    pub id: Uuid,
    pub rule_name: String,
    pub reaction_type: String,
    pub status: String,
    pub error: Option<String>,
    pub executed_at: i64,
}

pub async fn list_recent_executions(
    pool: &PgPool,
    team_id: Uuid,
) -> Result<Vec<ExecutionResponse>, AppError> {
    let rows = repo::rules::list_recent_executions(pool, team_id, 20).await?;
    Ok(rows
        .into_iter()
        .map(|r| ExecutionResponse {
            id: r.id,
            rule_name: r.rule_name,
            reaction_type: r.reaction_type,
            status: r.status,
            error: r.error,
            executed_at: r.executed_at.timestamp(),
        })
        .collect())
}
