pub mod auth;
pub mod incidents;
pub mod releases;
pub mod teams;
pub mod ws;

use crate::state::AppState;
use axum::{Router, routing::get};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(crate::handlers::health::check))
        .merge(auth::routes())
        .merge(teams::routes())
        .merge(incidents::routes())
        .merge(ws::routes())
        .merge(releases::routes())
}
