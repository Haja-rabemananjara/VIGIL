use axum::{Router, routing::post};

use crate::AppState;
use crate::handlers;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/auth/signup", post(handlers::auth::signup))
        .route("/auth/signin", post(handlers::auth::signin))
}
