pub mod about;
pub mod auth;
pub mod incidents;
pub mod messages;
pub mod reactions;
pub mod releases;
pub mod rules;
pub mod service_connections;
pub mod teams;
pub mod webhooks;
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
        .merge(rules::routes())
        .merge(webhooks::routes())
        .merge(service_connections::routes())
        .merge(about::routes())
        .merge(reactions::routes())
        .merge(messages::routes())
}
