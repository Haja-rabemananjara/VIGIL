use chrono::{Duration, Utc};
use rand::RngCore;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use sqlx::PgPool;

use crate::error::AppError;
use crate::repo;
use crate::services::auth::hash_password;

#[derive(Debug, Deserialize)]
struct GitHubTokenResponse {
    access_token: String,
}

#[derive(Debug, Deserialize)]
struct GitHubUser {
    email: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitHubEmail {
    email: String,
    primary: bool,
    verified: bool,
}

pub fn github_authorize_url(client_id: &str, state: &str) -> String {
    format!(
        "https://github.com/login/oauth/authorize?client_id={}&state={}&scope=user:email",
        client_id, state
    )
}

pub async fn github_callback(
    pool: &PgPool,
    http_client: &reqwest::Client,
    client_id: &str,
    client_secret: &str,
    code: &str,
) -> Result<(String, repo::user::User), AppError> {
    let token_res = http_client
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .json(&serde_json::json!({
            "client_id": client_id,
            "client_secret": client_secret,
            "code": code,
        }))
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("GitHub token exchange failed: {e}")))?;

    if !token_res.status().is_success() {
        return Err(AppError::Unauthorized(
            "GitHub rejected the authorization code".into(),
        ));
    }

    let token_body: GitHubTokenResponse = token_res
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to parse GitHub token response: {e}")))?;

    let access_token = &token_body.access_token;

    let email = fetch_github_email(http_client, access_token).await?;

    let user = match repo::user::find_by_email(pool, &email).await? {
        Some(user) => user,
        None => {
            let random_password: String = hex::encode({
                let mut buf = [0u8; 32];
                rand::rng().fill_bytes(&mut buf);
                buf
            });
            let password_hash = hash_password(&random_password)?;

            let display_name = email.split('@').next().unwrap_or("User");
            repo::user::insert_user(pool, &email, &password_hash, display_name)
                .await
                .map_err(|e| AppError::Internal(format!("Failed to create user: {e}")))?
        }
    };

    let mut raw_token = [0u8; 32];
    rand::rng().fill_bytes(&mut raw_token);
    let token_hex = hex::encode(raw_token);
    let token_hash = Sha256::digest(raw_token).to_vec();
    let expires_at = Utc::now() + Duration::days(30);

    repo::session::insert_session(pool, user.id, &token_hash, expires_at).await?;

    Ok((token_hex, user))
}

async fn fetch_github_email(
    http_client: &reqwest::Client,
    access_token: &str,
) -> Result<String, AppError> {
    let user_res: GitHubUser = http_client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {access_token}"))
        .header("User-Agent", "VIGIL")
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("GitHub user fetch failed: {e}")))?
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to parse GitHub user: {e}")))?;

    if let Some(email) = user_res.email {
        return Ok(email.to_lowercase());
    }

    let emails: Vec<GitHubEmail> = http_client
        .get("https://api.github.com/user/emails")
        .header("Authorization", format!("Bearer {access_token}"))
        .header("User-Agent", "VIGIL")
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("GitHub emails fetch failed: {e}")))?
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to parse GitHub emails: {e}")))?;

    emails
        .into_iter()
        .find(|e| e.primary && e.verified)
        .map(|e| e.email.to_lowercase())
        .ok_or_else(|| AppError::Unauthorized("No verified primary email on GitHub account".into()))
}
