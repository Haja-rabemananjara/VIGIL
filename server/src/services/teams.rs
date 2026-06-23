use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::team::{self, Role, TeamView};
use crate::error::AppError;
use crate::repo;

pub async fn create_team(
    pool: &PgPool,
    creator_id: Uuid,
    raw_name: &str,
) -> Result<TeamView, AppError> {
    let name =
        team::validate_team_name(raw_name).map_err(|msg| AppError::Validation(msg.to_string()))?;

    let team_id = Uuid::new_v4();
    let membership_id = Uuid::new_v4();

    let mut tx = pool.begin().await?;

    let created_at = repo::teams::insert_team(&mut tx, team_id, &name, creator_id).await?;
    repo::teams::insert_team_member(&mut tx, membership_id, team_id, creator_id, Role::Manager)
        .await?;

    tx.commit().await?;

    Ok(TeamView {
        id: team_id,
        name,
        created_at,
        role: Role::Manager,
    })
}

pub async fn list_my_teams(pool: &PgPool, user_id: Uuid) -> Result<Vec<TeamView>, AppError> {
    repo::teams::list_teams_for_user(pool, user_id).await
}

pub async fn get_team(pool: &PgPool, user_id: Uuid, team_id: Uuid) -> Result<TeamView, AppError> {
    repo::teams::find_team_for_member(pool, team_id, user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("team not found".into()))
}
