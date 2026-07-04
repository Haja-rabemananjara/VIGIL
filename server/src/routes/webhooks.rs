use crate::handlers;
use crate::state::AppState;
use axum::{Router, routing::post};

pub fn routes() -> Router<AppState> {
    Router::new().route("/webhooks/github", post(handlers::webhooks::receive_github))
}
