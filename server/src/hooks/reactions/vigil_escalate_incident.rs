use async_trait::async_trait;
use serde::Deserialize;
use uuid::Uuid;

use crate::error::AppError;
use crate::hooks::{ReactionContext, ReactionExecutor};
use crate::services::incidents;

pub struct VigilEscalateIncident;

impl VigilEscalateIncident {
    pub fn new() -> Self {
        Self
    }
}

impl Default for VigilEscalateIncident {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct EscalatePayload {
    incident_id: Uuid,
    #[serde(default)]
    severity: Option<String>,
}

#[async_trait]
impl ReactionExecutor for VigilEscalateIncident {
    fn kind(&self) -> &'static str {
        "vigil_escalate_incident"
    }

    fn service_name(&self) -> &'static str {
        "vigil"
    }

    fn description(&self) -> &'static str {
        "Escalate a VIGIL incident, optionally raising its severity"
    }

    fn payload_example(&self) -> &'static str {
        r#"{
  "incident_id": "00000000-0000-0000-0000-000000000000",
  "severity": "critical"
}"#
    }

    async fn execute(&self, ctx: &ReactionContext<'_>) -> Result<(), AppError> {
        let payload: EscalatePayload =
            serde_json::from_value(ctx.payload.clone()).map_err(|e| {
                AppError::Validation(format!("Invalid vigil_escalate_incident payload: {e}"))
            })?;

        let _incident = incidents::transition_incident_status(
            ctx.pool,
            ctx.broadcaster.clone(),
            payload.incident_id,
            ctx.team_id,
            ctx.rule_created_by,
            "escalated".to_string(),
            payload.severity,
        )
        .await?;

        Ok(())
    }
}
