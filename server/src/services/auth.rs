use chrono::{Duration, Utc};
use rand::RngCore;
use sha2::{Digest, Sha256};
use sqlx::PgPool;

use crate::domain::user::{self, normalize_email};
use crate::error::AppError;
use crate::repo;
use crate::repo::user::User;

pub async fn signup(
    pool: &PgPool,
    email: &str,
    password: &str,
    display_name: &str,
) -> Result<User, AppError> {
    user::validate_signup(email, password, display_name).map_err(AppError::Validation)?;

    let email = normalize_email(email);
    let display_name = display_name.trim();

    let password_hash = hash_password(password)?;

    match repo::user::insert_user(pool, &email, &password_hash, display_name).await {
        Ok(user) => Ok(user),
        Err(sqlx::Error::Database(db)) if db.is_unique_violation() => {
            Err(AppError::Conflict("email already in use".to_string()))
        }
        Err(e) => {
            tracing::error!(error = ?e, "insert_user failed");
            Err(e.into())
        }
    }
}

fn hash_password(password: &str) -> Result<String, AppError> {
    use argon2::Argon2;
    use argon2::password_hash::{PasswordHasher, SaltString, rand_core::OsRng};

    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|_| AppError::Internal("password hashing failed".to_string()))
}

pub async fn signin(
    pool: &PgPool,
    email: &str,
    password: &str,
) -> Result<(String, User), AppError> {
    let email = normalize_email(email);

    let user = repo::user::find_by_email(pool, &email)
        .await?
        .ok_or(AppError::Unauthorized("invalid credentials".to_string()))?;

    verify_password(password, &user.password_hash)?;

    let mut raw_token = [0u8; 32];
    rand::rng().fill_bytes(&mut raw_token);
    let token_hex = hex::encode(raw_token);

    let token_hash = Sha256::digest(raw_token).to_vec();

    let expires_at = Utc::now() + Duration::days(30);
    repo::session::insert_session(pool, user.id, &token_hash, expires_at).await?;

    Ok((token_hex, user))
}

fn verify_password(password: &str, hash: &str) -> Result<(), AppError> {
    use argon2::password_hash::PasswordVerifier;
    use argon2::{Argon2, PasswordHash};

    let parsed = PasswordHash::new(hash)
        .map_err(|_| AppError::Internal("invalid stored hash".to_string()))?;

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .map_err(|_| AppError::Unauthorized("invalid credentials".to_string()))
}
