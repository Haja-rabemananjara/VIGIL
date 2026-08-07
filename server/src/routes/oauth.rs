use crate::AppState;
use crate::handlers;
use axum::{Router, routing::get};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/auth/oauth/github", get(handlers::oauth::github_redirect))
        .route(
            "/auth/oauth/github/callback",
            get(handlers::oauth::github_callback),
        )
}
