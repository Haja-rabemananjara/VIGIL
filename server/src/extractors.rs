use std::collections::HashMap;

use axum::{
    extract::{FromRequestParts, Path},
    http::request::Parts,
};
use uuid::Uuid;

use crate::domain::team::Role;
use crate::error::AppError;
use crate::handlers::auth::AuthUser;
use crate::repo;
use crate::state::AppState;

pub struct TeamMember {
    pub team_id: Uuid,
    pub user_id: Uuid,
    pub role: Role,
}

impl FromRequestParts<AppState> for TeamMember {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let user = AuthUser::from_request_parts(parts, state).await?;

        let Path(params) = Path::<HashMap<String, String>>::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::NotFound("not found".into()))?;

        let team_id = params
            .get("team_id")
            .and_then(|s| Uuid::parse_str(s).ok())
            .ok_or_else(|| AppError::NotFound("not found".into()))?;

        let row = repo::teams::find_membership(&state.pool, team_id, user.id)
            .await?
            .ok_or_else(|| AppError::NotFound("not found".into()))?;

        let role = Role::from_db(&row.role)
            .ok_or_else(|| AppError::Internal(format!("unknown role: {}", row.role)))?;

        Ok(TeamMember {
            team_id,
            user_id: user.id,
            role,
        })
    }
}

pub struct RequireResponder(pub TeamMember);

impl FromRequestParts<AppState> for RequireResponder {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let member = TeamMember::from_request_parts(parts, state).await?;

        if !member.role.has_at_least(Role::Responder) {
            return Err(AppError::Forbidden("insufficient role".into()));
        }

        Ok(RequireResponder(member))
    }
}

pub struct RequireManager(pub TeamMember);

impl FromRequestParts<AppState> for RequireManager {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let member = TeamMember::from_request_parts(parts, state).await?;

        if !member.role.has_at_least(Role::Manager) {
            return Err(AppError::Forbidden("insufficient role".into()));
        }

        Ok(RequireManager(member))
    }
}
