use chrono::{DateTime, Utc};
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use crate::error::AppError;

pub struct InvitationRow {
    pub id: Uuid,
    pub team_id: Uuid,
    pub code: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub max_uses: Option<i32>,
    pub uses: i32,
}

pub struct MembershipStatus {
    pub id: Uuid,
    pub status: String,
}

pub async fn insert_invitation(
    conn: &mut PgConnection,
    id: Uuid,
    team_id: Uuid,
    code: &str,
    created_by: Uuid,
    expires_at: Option<DateTime<Utc>>,
    max_uses: Option<i32>,
) -> Result<InvitationRow, AppError> {
    let row = sqlx::query_as!(
        InvitationRow,
        r#"
        INSERT INTO invitations (id, team_id, code, created_by, expires_at, max_uses)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, team_id, code, created_by, created_at,
                  expires_at, max_uses, uses
        "#,
        id,
        team_id,
        code,
        created_by,
        expires_at,
        max_uses,
    )
    .fetch_one(conn)
    .await?;

    Ok(row)
}

pub async fn find_valid_invitation_by_code(
    pool: &PgPool,
    code: &str,
) -> Result<Option<InvitationRow>, AppError> {
    let row = sqlx::query_as!(
        InvitationRow,
        r#"
        SELECT id, team_id, code, created_by, created_at,
               expires_at, max_uses, uses
        FROM invitations
        WHERE code = $1
          AND status = 'active'
          AND (expires_at IS NULL OR expires_at > now())
          AND (max_uses IS NULL OR uses < max_uses)
        "#,
        code,
    )
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn invitation_code_ever_existed(pool: &PgPool, code: &str) -> Result<bool, AppError> {
    let row = sqlx::query_scalar!(
        r#"SELECT EXISTS(SELECT 1 FROM invitations WHERE code = $1) AS "exists!""#,
        code,
    )
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn increment_invitation_uses(
    conn: &mut PgConnection,
    invitation_id: Uuid,
) -> Result<(), AppError> {
    sqlx::query!(
        "UPDATE invitations SET uses = uses + 1 WHERE id = $1",
        invitation_id,
    )
    .execute(conn)
    .await?;

    Ok(())
}

pub async fn is_user_banned(pool: &PgPool, team_id: Uuid, user_id: Uuid) -> Result<bool, AppError> {
    let banned = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM team_bans
            WHERE team_id = $1
              AND user_id = $2
              AND status = 'active'
              AND (expires_at IS NULL OR expires_at > now())
        ) AS "exists!"
        "#,
        team_id,
        user_id,
    )
    .fetch_one(pool)
    .await?;

    Ok(banned)
}

pub async fn find_any_membership(
    conn: &mut PgConnection,
    team_id: Uuid,
    user_id: Uuid,
) -> Result<Option<MembershipStatus>, AppError> {
    let row = sqlx::query_as!(
        MembershipStatus,
        r#"
        SELECT id, status
        FROM team_members
        WHERE team_id = $1 AND user_id = $2
        "#,
        team_id,
        user_id,
    )
    .fetch_optional(conn)
    .await?;

    Ok(row)
}

pub async fn reactivate_member(
    conn: &mut PgConnection,
    membership_id: Uuid,
) -> Result<(), AppError> {
    sqlx::query!(
        r#"
        UPDATE team_members
        SET status = 'active', role = 'observer', joined_at = now()
        WHERE id = $1
        "#,
        membership_id,
    )
    .execute(conn)
    .await?;

    Ok(())
}
