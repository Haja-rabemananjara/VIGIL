use crate::crypto::KEY_LEN;
use crate::ws::{Broadcaster, presence::PresenceTracker};
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub broadcaster: Broadcaster,
    pub presence: PresenceTracker,
    pub webhook_secret: String,
    pub master_key: [u8; KEY_LEN],
}
