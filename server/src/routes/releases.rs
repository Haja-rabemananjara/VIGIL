use axum::Router;
use axum::routing::get;

use crate::{handlers, state::AppState};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/teams/{team_id}/releases",
            get(handlers::releases::list).post(handlers::releases::create),
        )
        .route(
            "/teams/{team_id}/releases/{release_id}",
            get(handlers::releases::get),
        )
}
