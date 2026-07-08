use sqlx::PgPool;
use uuid::Uuid;

use crate::crypto::{self, KEY_LEN};
use crate::domain::service_connections::{ServiceConnection, ServiceName};
use crate::error::AppError;
use crate::repo;

pub fn parse_service_name(s: &str) -> Result<ServiceName, AppError> {
    ServiceName::from_db(s).ok_or_else(|| AppError::NotFound(format!("Unknown service: {s}")))
}

pub async fn connect_service(
    pool: &PgPool,
    master_key: &[u8; KEY_LEN],
    user_id: Uuid,
    service: ServiceName,
    plaintext_token: &str,
) -> Result<ServiceConnection, AppError> {
    let trimmed = plaintext_token.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation("Token cannot be empty".to_string()));
    }

    let encrypted = crypto::encrypt(master_key, trimmed.as_bytes())?;

    let connection =
        repo::service_connections::upsert_connection(pool, user_id, service, &encrypted).await?;

    Ok(connection)
}

pub async fn list_connections(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<ServiceConnection>, AppError> {
    Ok(repo::service_connections::list_by_user(pool, user_id).await?)
}

pub async fn disconnect_service(
    pool: &PgPool,
    user_id: Uuid,
    service: ServiceName,
) -> Result<(), AppError> {
    let deleted = repo::service_connections::delete_connection(pool, user_id, service).await?;
    if !deleted {
        return Err(AppError::NotFound("Service not connected".to_string()));
    }
    Ok(())
}
