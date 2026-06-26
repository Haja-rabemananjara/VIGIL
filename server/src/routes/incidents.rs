use axum::{
    Router,
    routing::{get, post},
};

use crate::{handlers, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/teams/{team_id}/incidents",
            post(handlers::incidents::create_incident).get(handlers::incidents::list_incidents),
        )
        .route(
            "/teams/{team_id}/incidents/{incident_id}",
            get(handlers::incidents::get_incident),
        )
}
