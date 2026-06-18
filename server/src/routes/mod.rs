pub mod auth;

use crate::handlers;
use crate::state::AppState;
use axum::{Router, routing::get};

pub fn router() -> Router<AppState> {
    Router::new().route("/health", get(handlers::health::check))
}
