use axum::{routing::get, Router};
use crate::handlers;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(handlers::health::check))
}
