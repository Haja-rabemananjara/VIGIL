use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::invitation::generate_invite_code;
use crate::domain::team::Role;
use crate::error::AppError;
use crate::repo;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct InvitationView {
    pub id: Uuid,
    pub code: String,
    pub expires_at: Option<String>,
    pub max_uses: Option<i32>,
    pub uses: i32,
}

#[derive(Debug, Serialize)]
pub struct JoinResult {
    pub team_id: Uuid,
    pub team_name: String,
    pub role: String,
}

pub async fn create_invitation(
    pool: &PgPool,
    team_id: Uuid,
    created_by: Uuid,
) -> Result<InvitationView, AppError> {
    let id = Uuid::new_v4();
    let code = generate_invite_code();
    let expires_at = Some(Utc::now() + Duration::days(7));

    let mut conn = pool.acquire().await?;
    let row = repo::invitations::insert_invitation(
        &mut conn, id, team_id, &code, created_by, expires_at, None,
    )
    .await?;

    Ok(InvitationView {
        id: row.id,
        code: row.code,
        expires_at: row.expires_at.map(|t| t.to_rfc3339()),
        max_uses: row.max_uses,
        uses: row.uses,
    })
}

pub async fn join_team(pool: &PgPool, user_id: Uuid, code: &str) -> Result<JoinResult, AppError> {
    let invitation = match repo::invitations::find_valid_invitation_by_code(pool, code).await? {
        Some(inv) => inv,
        None => {
            let existed = repo::invitations::invitation_code_ever_existed(pool, code).await?;
            if existed {
                return Err(AppError::Gone("invitation code has expired".into()));
            }
            return Err(AppError::NotFound("invitation code not found".into()));
        }
    };

    let team_id = invitation.team_id;

    if repo::invitations::is_user_banned(pool, team_id, user_id).await? {
        return Err(AppError::Forbidden("you are banned from this team".into()));
    }

    let mut tx = pool.begin().await?;

    let existing = repo::invitations::find_any_membership(&mut tx, team_id, user_id).await?;

    if let Some(row) = existing {
        match row.status.as_str() {
            "active" => {
                return Err(AppError::Conflict("already a member of this team".into()));
            }
            "kicked" => {
                repo::invitations::reactivate_member(&mut tx, row.id).await?;
            }
            other => {
                return Err(AppError::Internal(format!(
                    "unexpected member status: {other}"
                )));
            }
        }
    } else {
        repo::teams::insert_team_member(&mut tx, Uuid::new_v4(), team_id, user_id, Role::Observer)
            .await?;
    }

    repo::invitations::increment_invitation_uses(&mut tx, invitation.id).await?;

    tx.commit().await?;

    let team = repo::teams::find_team_by_id(pool, team_id)
        .await?
        .ok_or_else(|| AppError::Internal("team not found after join".into()))?;

    Ok(JoinResult {
        team_id,
        team_name: team.name,
        role: "observer".into(),
    })
}
