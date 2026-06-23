pub mod auth;
pub mod teams;

use crate::state::AppState;
use axum::{Router, routing::get};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(crate::handlers::health::check))
        .merge(auth::routes())
        .merge(teams::router())
}
