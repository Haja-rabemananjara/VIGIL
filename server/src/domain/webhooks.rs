use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct WebhookDelivery {
    pub id: Uuid,
    pub service: String,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub headers: Option<serde_json::Value>,
    pub source: Option<String>,
    pub hmac_valid: Option<bool>,
    pub received_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
}
