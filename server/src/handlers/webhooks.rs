use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;

use crate::error::AppError;
use crate::services;
use crate::state::AppState;

pub async fn receive_github(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse, AppError> {
    let signature = headers
        .get("x-hub-signature-256")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("Missing X-Hub-Signature-256 header".to_string()))?;

    services::webhooks::verify_github_signature(&state.webhook_secret, &body, signature)?;

    let payload: serde_json::Value = serde_json::from_slice(&body)
        .map_err(|e| AppError::Validation(format!("Invalid JSON body: {e}")))?;

    let event_type = headers
        .get("x-github-event")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    let headers_json = serialize_headers(&headers);

    let delivery_id = services::webhooks::persist_delivery(
        &state.pool,
        services::webhooks::IncomingDelivery {
            service: "github",
            event_type,
            payload: &payload,
            headers: Some(&headers_json),
            source: None,
            hmac_valid: true,
        },
    )
    .await?;

    let pool = state.pool.clone();
    let event_type_owned = event_type.to_string();
    let payload_clone = payload.clone();
    tokio::spawn(async move {
        services::webhooks::process_delivery(
            &pool,
            delivery_id,
            "github",
            &event_type_owned,
            &payload_clone,
        )
        .await;
    });

    Ok(StatusCode::ACCEPTED)
}

fn serialize_headers(headers: &HeaderMap) -> serde_json::Value {
    let map: serde_json::Map<String, serde_json::Value> = headers
        .iter()
        .filter_map(|(name, value)| {
            value
                .to_str()
                .ok()
                .map(|v| (name.to_string(), serde_json::Value::String(v.to_string())))
        })
        .collect();
    serde_json::Value::Object(map)
}
