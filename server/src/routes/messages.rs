use axum::{Router, routing::post};

use crate::{handlers, state::AppState};

pub fn routes() -> Router<AppState> {
    Router::new().route(
        "/messages/{user_id}",
        post(handlers::messages::send_message).get(handlers::messages::get_conversation),
    )
}
