use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::service_connections::{
    ServiceConnection, ServiceConnectionWithToken, ServiceName,
};

pub async fn upsert_connection(
    pool: &PgPool,
    user_id: Uuid,
    service: ServiceName,
    encrypted_token: &[u8],
) -> Result<ServiceConnection, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        INSERT INTO service_connections (id, user_id, service, encrypted_token)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (user_id, service) DO UPDATE
            SET encrypted_token = EXCLUDED.encrypted_token,
                updated_at = now()
        RETURNING id, service, created_at, updated_at
        "#,
        Uuid::new_v4(),
        user_id,
        service.as_str(),
        encrypted_token,
    )
    .fetch_one(pool)
    .await?;

    let service = ServiceName::from_db(&row.service).ok_or_else(|| {
        sqlx::Error::Decode(format!("Unknown service in DB: {}", row.service).into())
    })?;

    Ok(ServiceConnection {
        id: row.id,
        service,
        created_at: row.created_at.timestamp(),
        updated_at: row.updated_at.timestamp(),
    })
}

pub async fn list_by_user(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<ServiceConnection>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT id, service, created_at, updated_at
        FROM service_connections
        WHERE user_id = $1
        ORDER BY service ASC
        "#,
        user_id,
    )
    .fetch_all(pool)
    .await?;

    let mut connections = Vec::with_capacity(rows.len());
    for row in rows {
        let service = ServiceName::from_db(&row.service).ok_or_else(|| {
            sqlx::Error::Decode(format!("Unknown service in DB: {}", row.service).into())
        })?;
        connections.push(ServiceConnection {
            id: row.id,
            service,
            created_at: row.created_at.timestamp(),
            updated_at: row.updated_at.timestamp(),
        });
    }

    Ok(connections)
}

pub async fn find_with_token(
    pool: &PgPool,
    user_id: Uuid,
    service: ServiceName,
) -> Result<Option<ServiceConnectionWithToken>, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT id, user_id, service, encrypted_token, created_at, updated_at
        FROM service_connections
        WHERE user_id = $1 AND service = $2
        "#,
        user_id,
        service.as_str(),
    )
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let service = ServiceName::from_db(&row.service).ok_or_else(|| {
        sqlx::Error::Decode(format!("Unknown service in DB: {}", row.service).into())
    })?;

    Ok(Some(ServiceConnectionWithToken {
        id: row.id,
        user_id: row.user_id,
        service,
        encrypted_token: row.encrypted_token,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }))
}

pub async fn delete_connection(
    pool: &PgPool,
    user_id: Uuid,
    service: ServiceName,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"DELETE FROM service_connections WHERE user_id = $1 AND service = $2"#,
        user_id,
        service.as_str(),
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}
