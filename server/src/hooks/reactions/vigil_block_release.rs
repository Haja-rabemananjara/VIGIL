use async_trait::async_trait;
use serde::Deserialize;
use uuid::Uuid;

use crate::error::AppError;
use crate::hooks::{ReactionContext, ReactionExecutor};
use crate::services::releases;

pub struct VigilBlockRelease;

impl VigilBlockRelease {
    pub fn new() -> Self {
        Self
    }
}

impl Default for VigilBlockRelease {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct BlockReleasePayload {
    release_id: Uuid,
    incident_id: Uuid,
}

#[async_trait]
impl ReactionExecutor for VigilBlockRelease {
    fn kind(&self) -> &'static str {
        "vigil_block_release"
    }

    fn description(&self) -> &'static str {
        "Block a VIGIL release by linking an active incident to it"
    }

    async fn execute(&self, ctx: &ReactionContext<'_>) -> Result<(), AppError> {
        let payload: BlockReleasePayload =
            serde_json::from_value(ctx.payload.clone()).map_err(|e| {
                AppError::Validation(format!("Invalid vigil_block_release payload: {e}"))
            })?;

        let _release = releases::link_incident(
            ctx.pool,
            ctx.broadcaster.clone(),
            payload.release_id,
            payload.incident_id,
            ctx.team_id,
            ctx.rule_created_by,
        )
        .await?;

        Ok(())
    }
}
