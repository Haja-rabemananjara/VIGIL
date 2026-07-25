use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;

pub struct MessageRow {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub recipient_id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

pub async fn insert_message(
    pool: &PgPool,
    id: Uuid,
    sender_id: Uuid,
    recipient_id: Uuid,
    content: &str,
) -> Result<MessageRow, AppError> {
    let row = sqlx::query_as!(
        MessageRow,
        r#"
        INSERT INTO private_messages (id, sender_id, recipient_id, content, created_at)
        VALUES ($1, $2, $3, $4, now())
        RETURNING id, sender_id, recipient_id, content, created_at
        "#,
        id,
        sender_id,
        recipient_id,
        content,
    )
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn list_conversation(
    pool: &PgPool,
    user_a: Uuid,
    user_b: Uuid,
    before: Option<DateTime<Utc>>,
    limit: i64,
) -> Result<Vec<MessageRow>, AppError> {
    let rows = sqlx::query_as!(
        MessageRow,
        r#"
        SELECT id, sender_id, recipient_id, content, created_at
        FROM private_messages
        WHERE LEAST(sender_id, recipient_id) = LEAST($1::uuid, $2::uuid)
          AND GREATEST(sender_id, recipient_id) = GREATEST($1::uuid, $2::uuid)
          AND ($3::timestamptz IS NULL OR created_at < $3)
        ORDER BY created_at ASC
        LIMIT $4
        "#,
        user_a,
        user_b,
        before,
        limit,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn share_a_team(pool: &PgPool, user_a: Uuid, user_b: Uuid) -> Result<bool, AppError> {
    let row = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM team_members a
            JOIN team_members b ON a.team_id = b.team_id
            WHERE a.user_id = $1
              AND b.user_id = $2
              AND a.status = 'active'
              AND b.status = 'active'
        ) AS "exists!"
        "#,
        user_a,
        user_b,
    )
    .fetch_one(pool)
    .await?;

    Ok(row)
}
