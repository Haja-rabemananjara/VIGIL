use chrono::{DateTime, Utc};
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use crate::domain::team::{Role, TeamView};
use crate::error::AppError;

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
