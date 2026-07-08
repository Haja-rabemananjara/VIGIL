use axum::{Router, routing::get};

use crate::handlers;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/about.json", get(handlers::about::get_about))
}
