use axum::{
    Router,
    routing::{get, post},
};

use crate::handlers;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/teams/{team_id}/rules",
            post(handlers::rules::create).get(handlers::rules::list),
        )
        .route(
            "/teams/{team_id}/rules/{rule_id}",
            get(handlers::rules::get_one)
                .patch(handlers::rules::update)
                .delete(handlers::rules::delete),
        )
        .route(
            "/teams/{team_id}/rules/executions",
            get(handlers::rules::list_executions),
        )
}
