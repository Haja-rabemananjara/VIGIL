use crate::ws::{Broadcaster, presence::PresenceTracker};
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub broadcaster: Broadcaster,
    pub presence: PresenceTracker,
    pub webhook_secret: String,
}
