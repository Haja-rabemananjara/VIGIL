use axum::{Router, routing::get};

use crate::handlers;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/teams/{team_id}/connections",
            get(handlers::team_connections::list),
        )
        .route(
            "/teams/{team_id}/connections/{service}",
            axum::routing::post(handlers::team_connections::connect)
                .delete(handlers::team_connections::disconnect),
        )
}
