use crate::AppState;
use crate::handlers;
use axum::{Router, routing::get};

pub fn routes() -> Router<AppState> {
    Router::new().route(
        "/teams/{team_id}/audit",
        get(handlers::audit::list_audit_log),
    )
}
