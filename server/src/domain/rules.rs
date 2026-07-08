use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Rule {
    pub id: Uuid,
    pub team_id: Uuid,
    pub name: String,
    pub enabled: bool,
    pub trigger_service: String,
    pub trigger_event: String,
    pub trigger_filters: serde_json::Value,
    pub reaction_type: String,
    pub reaction_payload: serde_json::Value,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRuleInput {
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub trigger: TriggerInput,
    pub reaction: ReactionInput,
}

#[derive(Debug, Deserialize)]
pub struct TriggerInput {
    pub service: String,
    pub event: String,
    #[serde(default)]
    pub filters: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct ReactionInput {
    #[serde(rename = "type")]
    pub reaction_type: String,
    #[serde(default)]
    pub payload: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRuleInput {
    pub name: Option<String>,
    pub enabled: Option<bool>,
    pub trigger: Option<TriggerInput>,
    pub reaction: Option<ReactionInput>,
}

fn default_true() -> bool {
    true
}
