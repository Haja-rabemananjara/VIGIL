use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::repo;
use crate::repo::user::User;
use crate::services::auth::hash_password;

pub struct UpdateProfileRequest {
    pub display_name: Option<String>,
    pub password: Option<String>,
    pub language: Option<String>,
    pub avatar_seed: Option<Option<String>>,
}

pub async fn update_profile(
    pool: &PgPool,
    user_id: Uuid,
    req: UpdateProfileRequest,
) -> Result<User, AppError> {
    if let Some(ref name) = req.display_name {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(AppError::Validation(
                "display name cannot be empty".to_string(),
            ));
        }
    }

    if let Some(ref lang) = req.language
        && lang != "en"
        && lang != "fr"
    {
        return Err(AppError::Validation(
            "language must be 'en' or 'fr'".to_string(),
        ));
    }

    if let Some(ref pw) = req.password
        && pw.len() < 8
    {
        return Err(AppError::Validation(
            "password must be at least 8 characters".to_string(),
        ));
    }

    let password_hash = match &req.password {
        Some(pw) => Some(hash_password(pw)?),
        None => None,
    };

    let display_name = req.display_name.as_deref().map(|n| n.trim());

    let avatar_seed = req.avatar_seed.as_ref().map(|opt| opt.as_deref());

    let user = repo::user::update_profile(
        pool,
        user_id,
        display_name,
        password_hash.as_deref(),
        req.language.as_deref(),
        avatar_seed,
    )
    .await
    .map_err(|e| {
        tracing::error!(error = ?e, "update_profile failed");
        AppError::Internal("failed to update profile".to_string())
    })?;

    Ok(user)
}
