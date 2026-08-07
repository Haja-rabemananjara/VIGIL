use crate::crypto::KEY_LEN;
use crate::hooks::{ActionCatalog, ReactionRegistry};
use crate::ws::{Broadcaster, presence::PresenceTracker};
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub broadcaster: Broadcaster,
    pub presence: PresenceTracker,
    pub webhook_secret: String,
    pub master_key: [u8; KEY_LEN],
    pub registry: ReactionRegistry,
    pub http_client: reqwest::Client,
    pub action_catalog: ActionCatalog,
    pub kickoff_token: String,
    pub github_client_id: Option<String>,
    pub github_client_secret: Option<String>,
}
