use axum::{
    Router,
    routing::{delete, get, post},
};

use crate::{handlers, state::AppState};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/reactions/available",
            get(handlers::reactions::get_available),
        )
        .route(
            "/teams/{team_id}/incidents/{incident_id}/reactions",
            get(handlers::reactions::get_incident_reactions),
        )
        .route(
            "/timeline/{entry_id}/reactions",
            post(handlers::reactions::add_reaction),
        )
        .route(
            "/timeline/{entry_id}/reactions/{emoji}",
            delete(handlers::reactions::remove_reaction),
        )
}
