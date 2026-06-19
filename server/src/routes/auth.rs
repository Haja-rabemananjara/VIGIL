use axum::{
    Router,
    routing::{get, post},
};

use crate::AppState;
use crate::handlers;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/auth/signup", post(handlers::auth::signup))
        .route("/auth/signin", post(handlers::auth::signin))
        .route("/me", get(handlers::auth::me))
}
