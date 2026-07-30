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
use crate::ws::{Broadcaster, WsEvent};

pub async fn create_rule(
    pool: &PgPool,
    broadcaster: Broadcaster,
    catalog: &ActionCatalog,
    registry: &ReactionRegistry,
    team_id: Uuid,
    actor_id: Uuid,
    input: CreateRuleInput,
) -> Result<Rule, AppError> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation(
            "Rule name cannot be empty".to_string(),
        ));
    }

    if !registry.contains(&input.reaction.reaction_type) {
        return Err(AppError::Validation(format!(
            "Unknown reaction: {}",
            input.reaction.reaction_type
        )));
    }

    if !catalog.contains(&input.trigger.service, &input.trigger.event) {
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
        pool,
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

    broadcaster
        .to_team(
            team_id,
            WsEvent::RuleCreated {
                team_id,
                rule_id: rule.id,
            },
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
    pool: &PgPool,
    broadcaster: Broadcaster,
    catalog: &ActionCatalog,
    registry: &ReactionRegistry,
    team_id: Uuid,
    rule_id: Uuid,
    input: UpdateRuleInput,
) -> Result<Rule, AppError> {
    if let Some(trigger) = &input.trigger
        && !catalog.contains(&trigger.service, &trigger.event)
    {
        return Err(AppError::Validation(format!(
            "Unknown trigger: {}.{}",
            trigger.service, trigger.event
        )));
    }

    if let Some(reaction) = &input.reaction
        && !registry.contains(&reaction.reaction_type)
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

    let rule = repo::rules::update_rule(pool, team_id, rule_id, patch)
        .await?
        .ok_or_else(|| AppError::NotFound("Rule not found".to_string()))?;

    broadcaster
        .to_team(team_id, WsEvent::RuleUpdated { team_id, rule_id })
        .await;

    Ok(rule)
}

pub async fn delete_rule(
    pool: &PgPool,
    broadcaster: Broadcaster,
    team_id: Uuid,
    rule_id: Uuid,
) -> Result<(), AppError> {
    let deleted = repo::rules::delete_rule(pool, team_id, rule_id).await?;
    if !deleted {
        return Err(AppError::NotFound("Rule not found".to_string()));
    }
    broadcaster
        .to_team(team_id, WsEvent::RuleDeleted { team_id, rule_id })
        .await;

    Ok(())
}
