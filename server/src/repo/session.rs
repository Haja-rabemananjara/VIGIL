use sqlx::PgPool;
use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

pub async fn insert_session(
    pool: &PgPool,
    user_id: Uuid,
    token_hash: &[u8],
    expires_at: DateTime<Utc>,
) -> Result<Session, sqlx::Error> {
    let id = Uuid::new_v4();
    sqlx::query_as!(
        Session,
        r#"
        INSERT INTO sessions (id, user_id, token_hash, expires_at)
        VALUES ($1, $2, $3, $4)
        RETURNING id, user_id, token_hash, created_at, expires_at
        "#,
        id,
        user_id,
        token_hash,
        expires_at,
    )
    .fetch_one(pool)
    .await
}

pub async fn find_by_token_hash(
    pool: &PgPool,
    token_hash: &[u8],
) -> Result<Option<Session>, sqlx::Error> {
    sqlx::query_as!(
        Session,
        r#"SELECT id, user_id, token_hash, created_at, expires_at
           FROM sessions WHERE token_hash = $1"#,
        token_hash,
    )
    .fetch_optional(pool)
    .await
}
