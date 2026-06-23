use axum::{
    Router,
    routing::{get, post},
};

use crate::handlers;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/teams",
            post(handlers::teams::create_team).get(handlers::teams::list_teams),
        )
        .route("/teams/{team_id}", get(handlers::teams::get_team))
}
