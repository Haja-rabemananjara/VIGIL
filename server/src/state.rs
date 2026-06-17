use sqlx::PgPool;
use crate::ws::broadcaster::Broadcaster;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub broadcaster: Broadcaster,
}
