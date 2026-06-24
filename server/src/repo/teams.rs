use chrono::{DateTime, Utc};
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use crate::domain::team::{Role, TeamView};
use crate::error::AppError;

pub struct MembershipRow {
    pub team_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
}

pub async fn insert_team(
    conn: &mut PgConnection,
    id: Uuid,
    name: &str,
    created_by: Uuid,
) -> Result<DateTime<Utc>, AppError> {
    let row = sqlx::query!(
        r#"
        INSERT INTO teams (id, name, created_by)
        VALUES ($1, $2, $3)
        RETURNING created_at
        "#,
        id,
        name,
        created_by,
    )
    .fetch_one(conn)
    .await?;

    Ok(row.created_at)
}

pub async fn insert_team_member(
    conn: &mut PgConnection,
    id: Uuid,
    team_id: Uuid,
    user_id: Uuid,
    role: Role,
) -> Result<(), AppError> {
    sqlx::query!(
        r#"
        INSERT INTO team_members (id, team_id, user_id, role)
        VALUES ($1, $2, $3, $4)
        "#,
        id,
        team_id,
        user_id,
        role.as_str(),
    )
    .execute(conn)
    .await?;

    Ok(())
}

pub async fn list_teams_for_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<TeamView>, AppError> {
    let rows = sqlx::query!(
        r#"
        SELECT t.id AS "id!", t.name AS "name!",
               t.created_at AS "created_at!", tm.role AS "role!"
        FROM teams t
        JOIN team_members tm ON tm.team_id = t.id
        WHERE tm.user_id = $1
          AND tm.status = 'active'
        ORDER BY t.created_at
        "#,
        user_id,
    )
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|r| {
            let role = Role::from_db(&r.role)
                .ok_or_else(|| AppError::Internal(format!("unknown role in db: {}", r.role)))?;
            Ok(TeamView {
                id: r.id,
                name: r.name,
                created_at: r.created_at,
                role,
            })
        })
        .collect()
}

pub async fn find_team_for_member(
    pool: &PgPool,
    team_id: Uuid,
    user_id: Uuid,
) -> Result<Option<TeamView>, AppError> {
    let row = sqlx::query!(
        r#"
        SELECT t.id AS "id!", t.name AS "name!",
               t.created_at AS "created_at!", tm.role AS "role!"
        FROM teams t
        JOIN team_members tm ON tm.team_id = t.id
        WHERE t.id = $1
          AND tm.user_id = $2
          AND tm.status = 'active'
        "#,
        team_id,
        user_id,
    )
    .fetch_optional(pool)
    .await?;

    row.map(|r| {
        let role = Role::from_db(&r.role)
            .ok_or_else(|| AppError::Internal(format!("unknown role in db: {}", r.role)))?;
        Ok(TeamView {
            id: r.id,
            name: r.name,
            created_at: r.created_at,
            role,
        })
    })
    .transpose()
}

pub async fn find_membership(
    pool: &PgPool,
    team_id: Uuid,
    user_id: Uuid,
) -> Result<Option<MembershipRow>, AppError> {
    let row = sqlx::query_as!(
        MembershipRow,
        r#"
        SELECT team_id, user_id, role
        FROM team_members
        WHERE team_id = $1
          AND user_id = $2
          AND status = 'active'
        "#,
        team_id,
        user_id,
    )
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn find_team_by_id(pool: &PgPool, team_id: Uuid) -> Result<Option<TeamRow>, AppError> {
    let row = sqlx::query_as!(
        TeamRow,
        r#"
        SELECT id, name, created_at
        FROM teams
        WHERE id = $1
        "#,
        team_id,
    )
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub struct TeamRow {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

pub struct MemberRow {
    pub user_id: Uuid,
    pub display_name: String,
    pub email: String,
    pub role: String,
    pub joined_at: DateTime<Utc>,
}

pub async fn list_team_members(pool: &PgPool, team_id: Uuid) -> Result<Vec<MemberRow>, AppError> {
    let rows = sqlx::query_as!(
        MemberRow,
        r#"
        SELECT u.id AS "user_id!",
               u.display_name AS "display_name!",
               u.email AS "email!",
               tm.role AS "role!",
               tm.joined_at AS "joined_at!"
        FROM team_members tm
        JOIN users u ON u.id = tm.user_id
        WHERE tm.team_id = $1
          AND tm.status = 'active'
        ORDER BY tm.joined_at
        "#,
        team_id,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn update_member_role(
    pool: &PgPool,
    team_id: Uuid,
    user_id: Uuid,
    new_role: &str,
) -> Result<bool, AppError> {
    let result = sqlx::query!(
        r#"
        UPDATE team_members
        SET role = $3
        WHERE team_id = $1
          AND user_id = $2
          AND status = 'active'
        "#,
        team_id,
        user_id,
        new_role,
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}
