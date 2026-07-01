use axum::{Router, routing::get};

use crate::{state::AppState, ws::handler::ws_handler};

pub fn routes() -> Router<AppState> {
    Router::new().route("/ws", get(ws_handler))
}
