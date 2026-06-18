use crate::ws::broadcaster::Broadcaster;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub broadcaster: Broadcaster,
}
