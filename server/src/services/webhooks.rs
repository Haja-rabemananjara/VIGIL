use hmac::{Hmac, Mac};
use sha2::Sha256;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::repo::{self, webhooks::NewDelivery};

type HmacSha256 = Hmac<Sha256>;

pub fn verify_github_signature(
    secret: &str,
    body: &[u8],
    signature_header: &str,
) -> Result<(), AppError> {
    let hex_sig = signature_header
        .strip_prefix("sha256=")
        .ok_or_else(|| AppError::Unauthorized("Invalid signature format".to_string()))?;

    let sig_bytes = hex::decode(hex_sig)
        .map_err(|_| AppError::Unauthorized("Invalid hex in signature".to_string()))?;

    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|_| AppError::Internal("HMAC key error".to_string()))?;

    mac.update(body);

    mac.verify_slice(&sig_bytes)
        .map_err(|_| AppError::Unauthorized("HMAC signature mismatch".to_string()))?;

    Ok(())
}

pub struct IncomingDelivery<'a> {
    pub service: &'a str,
    pub event_type: &'a str,
    pub payload: &'a serde_json::Value,
    pub headers: Option<&'a serde_json::Value>,
    pub source: Option<&'a str>,
    pub hmac_valid: bool,
}

pub async fn persist_delivery(
    pool: &PgPool,
    incoming: IncomingDelivery<'_>,
) -> Result<Uuid, AppError> {
    let id = Uuid::new_v4();

    repo::webhooks::insert_delivery(
        pool,
        NewDelivery {
            id,
            service: incoming.service,
            event_type: incoming.event_type,
            payload: incoming.payload,
            headers: incoming.headers,
            source: incoming.source,
            hmac_valid: incoming.hmac_valid,
        },
    )
    .await?;

    Ok(id)
}

pub async fn process_delivery(
    pool: &PgPool,
    delivery_id: Uuid,
    _service: &str,
    _event_type: &str,
    _payload: &serde_json::Value,
) {
    tracing::info!(
        delivery_id = %delivery_id,
        "Processing webhook delivery (placeholder — VGL-073 will add rule matching)"
    );

    if let Err(e) = repo::webhooks::mark_processed(pool, delivery_id).await {
        tracing::error!(error = %e, "Failed to mark delivery as processed");
    }
}
