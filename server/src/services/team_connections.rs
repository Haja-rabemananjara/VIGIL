use sqlx::PgPool;
use uuid::Uuid;

use crate::crypto::{self, KEY_LEN};
use crate::domain::team_connections::TeamServiceConnection;
use crate::error::AppError;
use crate::repo;

fn validate_service(service: &str) -> Result<(), AppError> {
    match service {
        "github" | "discord" => Ok(()),
        _ => Err(AppError::Validation(format!("Unknown service: {service}"))),
    }
}

pub async fn connect_service(
    pool: &PgPool,
    master_key: &[u8; KEY_LEN],
    team_id: Uuid,
    service: &str,
    plaintext_token: &str,
    created_by: Uuid,
) -> Result<TeamServiceConnection, AppError> {
    validate_service(service)?;

    let trimmed = plaintext_token.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation("Token cannot be empty".to_string()));
    }

    let encrypted = crypto::encrypt(master_key, trimmed.as_bytes())?;

    let connection =
        repo::team_connections::upsert_connection(pool, team_id, service, &encrypted, created_by)
            .await?;

    Ok(connection)
}

pub async fn list_connections(
    pool: &PgPool,
    team_id: Uuid,
) -> Result<Vec<TeamServiceConnection>, AppError> {
    Ok(repo::team_connections::list_by_team(pool, team_id).await?)
}

pub async fn disconnect_service(
    pool: &PgPool,
    team_id: Uuid,
    service: &str,
) -> Result<(), AppError> {
    validate_service(service)?;

    let deleted = repo::team_connections::delete_connection(pool, team_id, service).await?;
    if !deleted {
        return Err(AppError::NotFound("Service not connected".to_string()));
    }
    Ok(())
}

pub fn webhook_url(connection_id: Uuid, server_host: &str) -> String {
    format!("{server_host}/webhooks/{connection_id}")
}
