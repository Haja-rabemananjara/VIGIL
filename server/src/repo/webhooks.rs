use sqlx::PgPool;
use uuid::Uuid;

pub struct NewDelivery<'a> {
    pub id: Uuid,
    pub service: &'a str,
    pub event_type: &'a str,
    pub payload: &'a serde_json::Value,
    pub headers: Option<&'a serde_json::Value>,
    pub source: Option<&'a str>,
    pub hmac_valid: bool,
}

pub async fn insert_delivery(pool: &PgPool, delivery: NewDelivery<'_>) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO webhook_deliveries (id, service, event_type, payload, headers, source, hmac_valid)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
        delivery.id,
        delivery.service,
        delivery.event_type,
        delivery.payload,
        delivery.headers,
        delivery.source,
        delivery.hmac_valid,
    )
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn mark_processed(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE webhook_deliveries SET processed_at = now() WHERE id = $1"#,
        id,
    )
    .execute(pool)
    .await?;

    Ok(())
}
