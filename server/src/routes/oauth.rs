use axum::{Router, routing::get};
use crate::AppState;
use crate::handlers;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/auth/oauth/github", get(handlers::oauth::github_redirect))
        .route(
            "/auth/oauth/github/callback",
            get(handlers::oauth::github_callback),
        )
}