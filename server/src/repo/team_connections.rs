use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::team_connections::{TeamConnectionWithToken, TeamServiceConnection};

pub async fn upsert_connection(
    pool: &PgPool,
    team_id: Uuid,
    service: &str,
    encrypted_token: &[u8],
    created_by: Uuid,
) -> Result<TeamServiceConnection, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        INSERT INTO team_service_connections (id, team_id, service, encrypted_token, created_by)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (team_id, service) DO UPDATE
            SET encrypted_token = EXCLUDED.encrypted_token,
                updated_at = now()
        RETURNING id, team_id, service, created_by, created_at, updated_at
        "#,
        Uuid::new_v4(),
        team_id,
        service,
        encrypted_token,
        created_by,
    )
    .fetch_one(pool)
    .await?;

    Ok(TeamServiceConnection {
        id: row.id,
        team_id: row.team_id,
        service: row.service,
        created_by: row.created_by,
        created_at: row.created_at.timestamp(),
        updated_at: row.updated_at.timestamp(),
    })
}

pub async fn list_by_team(
    pool: &PgPool,
    team_id: Uuid,
) -> Result<Vec<TeamServiceConnection>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT id, team_id, service, created_by, created_at, updated_at
        FROM team_service_connections
        WHERE team_id = $1
        ORDER BY service ASC
        "#,
        team_id,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| TeamServiceConnection {
            id: r.id,
            team_id: r.team_id,
            service: r.service,
            created_by: r.created_by,
            created_at: r.created_at.timestamp(),
            updated_at: r.updated_at.timestamp(),
        })
        .collect())
}

pub async fn get_with_token(
    pool: &PgPool,
    connection_id: Uuid,
) -> Result<Option<TeamConnectionWithToken>, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT id, team_id, service, encrypted_token
        FROM team_service_connections
        WHERE id = $1
        "#,
        connection_id,
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| TeamConnectionWithToken {
        id: r.id,
        team_id: r.team_id,
        service: r.service,
        encrypted_token: r.encrypted_token,
    }))
}

pub async fn get_by_team_service_with_token(
    pool: &PgPool,
    team_id: Uuid,
    service: &str,
) -> Result<Option<TeamConnectionWithToken>, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT id, team_id, service, encrypted_token
        FROM team_service_connections
        WHERE team_id = $1 AND service = $2
        "#,
        team_id,
        service,
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| TeamConnectionWithToken {
        id: r.id,
        team_id: r.team_id,
        service: r.service,
        encrypted_token: r.encrypted_token,
    }))
}

pub async fn delete_connection(
    pool: &PgPool,
    team_id: Uuid,
    service: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"DELETE FROM team_service_connections WHERE team_id = $1 AND service = $2"#,
        team_id,
        service,
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}
