use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use uuid::Uuid;

use crate::error::AppError;
use crate::repo;
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
        .ok_or_else(|| AppError::Unauthorized("Missing X-Hub-Signature-256 header".into()))?;

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
            connection_id: None,
        },
    )
    .await?;

    let pool = state.pool.clone();
    let broadcaster = state.broadcaster.clone();
    let registry = state.registry.clone();
    let http_client = state.http_client.clone();
    let master_key = state.master_key;
    let event_type_owned = event_type.to_string();
    let payload_clone = payload.clone();
    tokio::spawn(async move {
        let engine_ctx = crate::hooks::engine::EngineContext {
            pool: &pool,
            broadcaster: &broadcaster,
            registry: &registry,
            http_client: &http_client,
            master_key: &master_key,
        };
        services::webhooks::process_delivery(
            &engine_ctx,
            delivery_id,
            "github",
            &event_type_owned,
            &payload_clone,
            None,
        )
        .await;
    });

    Ok(StatusCode::ACCEPTED)
}

pub async fn receive_webhook(
    State(state): State<AppState>,
    Path(connection_id): Path<Uuid>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse, AppError> {
    let connection = repo::team_connections::get_with_token(&state.pool, connection_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Unknown webhook endpoint".into()))?;

    let secret_bytes = crate::crypto::decrypt(&state.master_key, &connection.encrypted_token)
        .map_err(|_| AppError::Internal("Failed to decrypt webhook secret".into()))?;

    let secret = String::from_utf8(secret_bytes)
        .map_err(|_| AppError::Internal("Webhook secret is not valid UTF-8".into()))?;

    let signature = headers
        .get("x-hub-signature-256")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("Missing X-Hub-Signature-256 header".into()))?;

    services::webhooks::verify_github_signature(&secret, &body, signature)?;

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
            service: &connection.service,
            event_type,
            payload: &payload,
            headers: Some(&headers_json),
            source: None,
            hmac_valid: true,
            connection_id: Some(connection_id),
        },
    )
    .await?;

    let team_id = connection.team_id;
    let service_owned = connection.service.clone();
    let pool = state.pool.clone();
    let broadcaster = state.broadcaster.clone();
    let registry = state.registry.clone();
    let http_client = state.http_client.clone();
    let master_key = state.master_key;
    let event_type_owned = event_type.to_string();
    let payload_clone = payload.clone();
    tokio::spawn(async move {
        let engine_ctx = crate::hooks::engine::EngineContext {
            pool: &pool,
            broadcaster: &broadcaster,
            registry: &registry,
            http_client: &http_client,
            master_key: &master_key,
        };
        services::webhooks::process_delivery(
            &engine_ctx,
            delivery_id,
            &service_owned,
            &event_type_owned,
            &payload_clone,
            Some(team_id),
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
