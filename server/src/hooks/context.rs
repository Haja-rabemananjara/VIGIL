use crate::crypto::KEY_LEN;
use crate::ws::broadcaster::Broadcaster;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

pub struct ReactionContext<'a> {
    pub pool: &'a PgPool,
    pub broadcaster: &'a Broadcaster,
    pub team_id: Uuid,
    pub rule_id: Uuid,
    pub rule_name: &'a str,
    pub rule_created_by: Uuid,
    pub payload: &'a Value,
    pub http_client: &'a reqwest::Client,
    pub master_key: &'a [u8; KEY_LEN],
}
