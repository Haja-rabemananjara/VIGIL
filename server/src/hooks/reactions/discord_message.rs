use async_trait::async_trait;
use serde::Deserialize;

use crate::crypto;
use crate::domain::service_connections::ServiceName;
use crate::error::AppError;
use crate::hooks::{ReactionContext, ReactionExecutor};
use crate::repo;

pub struct DiscordMessage;

impl DiscordMessage {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DiscordMessage {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct DiscordPayload {
    content: String,
    #[serde(default)]
    username: Option<String>,
}

#[async_trait]
impl ReactionExecutor for DiscordMessage {
    fn kind(&self) -> &'static str {
        "discord_message"
    }

    fn service_name(&self) -> &'static str {
        "discord"
    }

    fn description(&self) -> &'static str {
        "Send a message to a Discord channel via webhook"
    }

    async fn execute(&self, ctx: &ReactionContext<'_>) -> Result<(), AppError> {
        let payload: DiscordPayload = serde_json::from_value(ctx.payload.clone())
            .map_err(|e| AppError::Validation(format!("Invalid discord_message payload: {e}")))?;

        let content = payload.content.trim();
        if content.is_empty() {
            return Err(AppError::Validation(
                "Discord message content cannot be empty".to_string(),
            ));
        }

        let connection = repo::service_connections::find_with_token(
            ctx.pool,
            ctx.rule_created_by,
            ServiceName::Discord,
        )
        .await?
        .ok_or_else(|| {
            AppError::Validation(
                "The rule creator has no Discord connection configured".to_string(),
            )
        })?;

        let webhook_bytes = crypto::decrypt(ctx.master_key, &connection.encrypted_token)?;
        let webhook_url = std::str::from_utf8(&webhook_bytes).map_err(|_| {
            AppError::Internal("Discord webhook URL is not valid UTF-8".to_string())
        })?;

        let mut body = serde_json::json!({ "content": content });
        if let Some(username) = &payload.username {
            body["username"] = serde_json::Value::String(username.clone());
        }

        let response = ctx
            .http_client
            .post(webhook_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Discord request failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(AppError::Internal(format!(
                "Discord returned HTTP {status}"
            )));
        }

        Ok(())
    }
}
