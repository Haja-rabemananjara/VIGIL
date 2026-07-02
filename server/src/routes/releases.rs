use axum::Router;
use axum::routing::{get, post};

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
        .route(
            "/teams/{team_id}/releases/{release_id}/start",
            post(handlers::releases::start),
        )
        .route(
            "/teams/{team_id}/releases/{release_id}/steps/{step_id}/validate",
            post(handlers::releases::validate_step),
        )
        .route(
            "/teams/{team_id}/releases/{release_id}/cancel",
            post(handlers::releases::cancel),
        )
}
