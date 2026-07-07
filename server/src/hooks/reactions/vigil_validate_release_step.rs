use async_trait::async_trait;
use serde::Deserialize;
use uuid::Uuid;

use crate::error::AppError;
use crate::hooks::{ReactionContext, ReactionExecutor};
use crate::services::releases;

pub struct VigilValidateReleaseStep;

impl VigilValidateReleaseStep {
    pub fn new() -> Self {
        Self
    }
}

impl Default for VigilValidateReleaseStep {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct ValidateStepPayload {
    release_id: Uuid,
    step_id: Uuid,
}

#[async_trait]
impl ReactionExecutor for VigilValidateReleaseStep {
    fn kind(&self) -> &'static str {
        "vigil_validate_release_step"
    }

    fn description(&self) -> &'static str {
        "Validate a specific step of a VIGIL release (moves the release forward)"
    }

    async fn execute(&self, ctx: &ReactionContext<'_>) -> Result<(), AppError> {
        let payload: ValidateStepPayload =
            serde_json::from_value(ctx.payload.clone()).map_err(|e| {
                AppError::Validation(format!("Invalid vigil_validate_release_step payload: {e}"))
            })?;

        let _release = releases::validate_step(
            ctx.pool,
            ctx.broadcaster.clone(),
            payload.release_id,
            payload.step_id,
            ctx.team_id,
            ctx.rule_created_by,
        )
        .await?;

        Ok(())
    }
}
