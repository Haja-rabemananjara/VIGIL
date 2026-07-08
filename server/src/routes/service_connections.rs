use axum::{
    Router,
    routing::{get, post},
};

use crate::handlers;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/me/services", get(handlers::service_connections::list))
        .route(
            "/me/services/{service}",
            post(handlers::service_connections::connect)
                .delete(handlers::service_connections::disconnect),
        )
}
