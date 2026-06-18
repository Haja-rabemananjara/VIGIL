pub mod auth;

use crate::state::AppState;
use axum::{Router, routing::get};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(crate::handlers::health::check))
        .merge(auth::routes())
}
