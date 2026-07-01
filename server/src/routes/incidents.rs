use axum::{
    Router,
    routing::{get, patch, post},
};

use crate::{handlers, state::AppState};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/teams/{team_id}/incidents",
            post(handlers::incidents::create_incident).get(handlers::incidents::list_incidents),
        )
        .route(
            "/teams/{team_id}/incidents/{incident_id}",
            get(handlers::incidents::get_incident),
        )
        .route(
            "/teams/{team_id}/incidents/{incident_id}/status",
            patch(handlers::incidents::transition_incident_status),
        )
        .route(
            "/teams/{team_id}/incidents/{incident_id}/severity",
            patch(handlers::incidents::update_incident_severity),
        )
        .route(
            "/teams/{team_id}/incidents/{incident_id}/assign",
            post(handlers::incidents::assign_responder),
        )
        .route(
            "/teams/{team_id}/incidents/{incident_id}/timeline",
            post(handlers::incidents::add_timeline_entry).get(handlers::incidents::get_timeline),
        )
}
