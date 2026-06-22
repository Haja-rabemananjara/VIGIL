use axum::{Json, response::IntoResponse};
use serde_json::json;

pub async fn check() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
}
