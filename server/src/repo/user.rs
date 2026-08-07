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
    pub avatar_seed: Option<String>,
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
        RETURNING id, email, password_hash, display_name, language, avatar_seed, created_at
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
        r#"SELECT id, email, password_hash, display_name, language, avatar_seed, created_at
           FROM users WHERE email = $1"#,
        email,
    )
    .fetch_optional(pool)
    .await
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as!(
        User,
        r#"SELECT id, email, password_hash, display_name, language, avatar_seed, created_at
           FROM users WHERE id = $1"#,
        id,
    )
    .fetch_optional(pool)
    .await
}

pub async fn update_profile(
    pool: &PgPool,
    user_id: Uuid,
    display_name: Option<&str>,
    password_hash: Option<&str>,
    language: Option<&str>,
    avatar_seed: Option<Option<&str>>,
) -> Result<User, sqlx::Error> {
    sqlx::query_as!(
        User,
        r#"
        UPDATE users
        SET display_name = COALESCE($2, display_name),
            password_hash = COALESCE($3, password_hash),
            language = COALESCE($4, language),
            avatar_seed = CASE WHEN $6 THEN $5 ELSE avatar_seed END
        WHERE id = $1
        RETURNING id, email, password_hash, display_name, language, avatar_seed, created_at
        "#,
        user_id,
        display_name,
        password_hash,
        language,
        avatar_seed.flatten(),
        avatar_seed.is_some(),
    )
    .fetch_one(pool)
    .await
}
