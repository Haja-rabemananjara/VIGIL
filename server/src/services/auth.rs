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
