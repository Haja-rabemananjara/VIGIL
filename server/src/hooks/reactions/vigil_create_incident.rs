use async_trait::async_trait;
use serde::Deserialize;

use crate::error::AppError;
use crate::hooks::{ReactionContext, ReactionExecutor};
use crate::services::incidents::{self, CreateIncidentInput};

pub struct VigilCreateIncident;

impl VigilCreateIncident {
    pub fn new() -> Self {
        Self
    }
}

impl Default for VigilCreateIncident {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct IncidentPayload {
    title: String,
    severity: String,
    #[serde(default)]
    body: String,
}

#[async_trait]
impl ReactionExecutor for VigilCreateIncident {
    fn kind(&self) -> &'static str {
        "vigil_create_incident"
    }

    fn service_name(&self) -> &'static str {
        "vigil"
    }

    fn description(&self) -> &'static str {
        "Create a VIGIL incident with configurable title, severity, and body"
    }

    fn payload_example(&self) -> &'static str {
        r#"{
  "title": "CI broken on {{repository.name}}",
  "severity": "high",
  "body": "Workflow {{workflow_run.name}} failed"
}"#
    }

    async fn execute(&self, ctx: &ReactionContext<'_>) -> Result<(), AppError> {
        let payload: IncidentPayload =
            serde_json::from_value(ctx.payload.clone()).map_err(|e| {
                AppError::Validation(format!("Invalid vigil_create_incident payload: {e}"))
            })?;

        let _incident = incidents::create_incident(
            ctx.pool,
            ctx.broadcaster.clone(),
            ctx.team_id,
            ctx.rule_created_by,
            CreateIncidentInput {
                title: payload.title,
                body: payload.body,
                severity: payload.severity,
            },
        )
        .await?;

        Ok(())
    }
}
