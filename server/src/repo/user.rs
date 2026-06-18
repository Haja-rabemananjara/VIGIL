use sqlx::PgPool;
use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub display_name: String,
    pub language: String,
    pub created_at: DateTime<Utc>,
}

pub async fn insert_user(
    pool: &PgPool,
    email: &str,
    password_hash: &str,
    display_name: &str,
) -> Result<User, sqlx::Error> {
    let id = Uuid::new_v4();
    sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (id, email, password_hash, display_name)
        VALUES ($1, $2, $3, $4)
        RETURNING id, email, password_hash, display_name, language, created_at
        "#,
        id,
        email,
        password_hash,
        display_name,
    )
    .fetch_one(pool)
    .await
}

pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as!(
        User,
        r#"SELECT id, email, password_hash, display_name, language, created_at
           FROM users WHERE email = $1"#,
        email,
    )
    .fetch_optional(pool)
    .await
}
