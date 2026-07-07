use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::rules::Rule;
use crate::hooks::context::ReactionContext;
use crate::hooks::registry::ReactionRegistry;
use crate::hooks::{matcher, templating};
use crate::repo;
use crate::ws::broadcaster::Broadcaster;
use crate::ws::events::WsEvent;

pub async fn evaluate(
    pool: &PgPool,
    broadcaster: &Broadcaster,
    registry: &ReactionRegistry,
    service: &str,
    event: &str,
    payload: &Value,
    delivery_id: Uuid,
) {
    let rules = match repo::rules::list_matching_rules(pool, service, event).await {
        Ok(rules) => rules,
        Err(err) => {
            tracing::error!(
                delivery_id = %delivery_id,
                error = %err,
                "Failed to load matching rules - aborting evaluation"
            );
            return;
        }
    };

    if rules.is_empty() {
        tracing::debug!(
            delivery_id = %delivery_id,
            service = %service,
            event = %event,
            "No matching rules for this delivery"
        );
        return;
    }

    tracing::info!(
        delivery_id = %delivery_id,
        service = %service,
        event = %event,
        rules_count = rules.len(),
        "Evaluating rules"
    );

    for rule in rules {
        evaluate_one(pool, broadcaster, registry, &rule, payload).await;
    }
}

async fn evaluate_one(
    pool: &PgPool,
    broadcaster: &Broadcaster,
    registry: &ReactionRegistry,
    rule: &Rule,
    payload: &Value,
) {
    if !matcher::matches(payload, &rule.trigger_filters) {
        tracing::debug!(
            rule_id = %rule.id,
            rule_name = %rule.name,
            "Rule filters did not match"
        );
        return;
    }

    let Some(executor) = registry.get(&rule.reaction_type) else {
        let error = format!("Reaction '{}' is not registered", rule.reaction_type);
        tracing::warn!(
            rule_id = %rule.id,
            rule_name = %rule.name,
            reaction_type = %rule.reaction_type,
            "{error}"
        );
        broadcast_failed(broadcaster, rule, &error).await;
        return;
    };

    let rendered_payload = templating::render(&rule.reaction_payload, payload);

    let ctx = ReactionContext {
        pool,
        broadcaster,
        team_id: rule.team_id,
        rule_id: rule.id,
        rule_name: &rule.name,
        rule_created_by: rule.created_by,
        payload: &rendered_payload,
    };

    match executor.execute(&ctx).await {
        Ok(()) => {
            tracing::info!(
                rule_id = %rule.id,
                rule_name = %rule.name,
                reaction_type = %rule.reaction_type,
                "Rule executed successfully"
            );
            broadcast_triggered(broadcaster, rule).await;
        }
        Err(err) => {
            let error = err.to_string();
            tracing::warn!(
                rule_id = %rule.id,
                rule_name = %rule.name,
                reaction_type = %rule.reaction_type,
                error = %error,
                "Rule execution failed"
            );
            broadcast_failed(broadcaster, rule, &error).await;
        }
    }
}

async fn broadcast_triggered(broadcaster: &Broadcaster, rule: &Rule) {
    let event = WsEvent::RuleTriggered {
        team_id: rule.team_id,
        rule_id: rule.id,
        rule_name: rule.name.clone(),
        reaction_type: rule.reaction_type.clone(),
    };
    broadcaster.to_team(rule.team_id, event).await;
}

async fn broadcast_failed(broadcaster: &Broadcaster, rule: &Rule, error: &str) {
    let event = WsEvent::RuleFailed {
        team_id: rule.team_id,
        rule_id: rule.id,
        rule_name: rule.name.clone(),
        reaction_type: rule.reaction_type.clone(),
        error: error.to_string(),
    };
    broadcaster.to_team(rule.team_id, event).await;
}
