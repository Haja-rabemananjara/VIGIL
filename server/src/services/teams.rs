use crate::services::audit;
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct MemberView {
    pub user_id: Uuid,
    pub display_name: String,
    pub email: String,
    pub role: String,
    pub joined_at: String,
}

use crate::domain::team::{self, Role, TeamView};
use crate::error::AppError;
use crate::repo;
use crate::ws::{Broadcaster, WsEvent};

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

pub async fn get_team_as_member(
    pool: &PgPool,
    team_id: Uuid,
    role: Role,
) -> Result<TeamView, AppError> {
    let row = repo::teams::find_team_by_id(pool, team_id)
        .await?
        .ok_or_else(|| AppError::Internal("team vanished after membership check".into()))?;

    Ok(TeamView {
        id: row.id,
        name: row.name,
        created_at: row.created_at,
        role,
    })
}

pub async fn list_members(pool: &PgPool, team_id: Uuid) -> Result<Vec<MemberView>, AppError> {
    let rows = repo::teams::list_team_members(pool, team_id).await?;

    Ok(rows
        .into_iter()
        .map(|r| MemberView {
            user_id: r.user_id,
            display_name: r.display_name,
            email: r.email,
            role: r.role,
            joined_at: r.joined_at.to_rfc3339(),
        })
        .collect())
}

pub async fn change_member_role(
    pool: &PgPool,
    broadcaster: Broadcaster,
    manager_id: Uuid,
    team_id: Uuid,
    target_user_id: Uuid,
    new_role: Role,
) -> Result<(), AppError> {
    if manager_id == target_user_id {
        return Err(AppError::Validation("cannot change your own role".into()));
    }

    if new_role == Role::Manager {
        return Err(AppError::Validation(
            "use the transfer endpoint to assign the Manager role".into(),
        ));
    }

    let updated =
        repo::teams::update_member_role(pool, team_id, target_user_id, new_role.as_str()).await?;

    if !updated {
        return Err(AppError::NotFound("member not found".into()));
    }

    broadcaster
        .to_team(
            team_id,
            WsEvent::MemberRoleChanged {
                team_id,
                user_id: target_user_id,
                new_role: new_role.as_str().to_string(),
                by: manager_id,
            },
        )
        .await;

    audit::record(
        pool,
        team_id,
        manager_id,
        &format!("member_role_{}", new_role.as_str()),
        "team_member",
        target_user_id,
        json!({"new_role": new_role.as_str() }),
    )
    .await;

    Ok(())
}

pub async fn transfer_manager(
    pool: &PgPool,
    broadcaster: Broadcaster,
    team_id: Uuid,
    current_manager_id: Uuid,
    target_user_id: Uuid,
) -> Result<(), AppError> {
    if current_manager_id == target_user_id {
        return Err(AppError::Validation(
            "cannot transfer manager role to yourself".into(),
        ));
    }

    let target = repo::teams::find_membership(pool, team_id, target_user_id).await?;
    if target.is_none() {
        return Err(AppError::NotFound("target member not found".into()));
    }

    let mut tx = pool.begin().await?;

    repo::teams::update_member_role_tx(
        &mut tx,
        team_id,
        current_manager_id,
        Role::Responder.as_str(),
    )
    .await?;

    let promoted = repo::teams::update_member_role_tx(
        &mut tx,
        team_id,
        target_user_id,
        Role::Manager.as_str(),
    )
    .await?;

    if !promoted {
        return Err(AppError::Internal(
            "target member vanished during transfer".into(),
        ));
    }

    tx.commit().await?;

    broadcaster
        .to_team(
            team_id,
            WsEvent::MemberRoleChanged {
                team_id,
                user_id: current_manager_id,
                new_role: "responder".to_string(),
                by: current_manager_id,
            },
        )
        .await;

    broadcaster
        .to_team(
            team_id,
            WsEvent::MemberRoleChanged {
                team_id,
                user_id: target_user_id,
                new_role: "manager".to_string(),
                by: current_manager_id,
            },
        )
        .await;

    audit::record(
        pool,
        team_id,
        current_manager_id,
        "manager_transferred",
        "team_member",
        target_user_id,
        json!({}),
    )
    .await;

    Ok(())
}

pub async fn leave_team(
    pool: &PgPool,
    team_id: Uuid,
    user_id: Uuid,
    role: Role,
) -> Result<(), AppError> {
    if role == Role::Manager {
        return Err(AppError::Conflict(
            "transfer the manager role before leaving".into(),
        ));
    }

    let updated = repo::teams::deactivate_member(pool, team_id, user_id).await?;

    if !updated {
        return Err(AppError::Internal("member vanished during leave".into()));
    }

    Ok(())
}

async fn check_moderation_target(
    pool: &PgPool,
    team_id: Uuid,
    manager_id: Uuid,
    target_user_id: Uuid,
) -> Result<(), AppError> {
    if manager_id == target_user_id {
        return Err(AppError::Validation("cannot moderate yourself".into()));
    }

    let membership = repo::teams::find_membership(pool, team_id, target_user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("member not found".into()))?;

    let target_role = Role::from_db(membership.role.as_str())
        .ok_or_else(|| AppError::Internal("invalid role in database".into()))?;

    if target_role == Role::Manager {
        return Err(AppError::Validation(
            "cannot kick or ban the current Manager; transfer the role first".into(),
        ));
    }

    Ok(())
}

pub async fn kick_member(
    pool: &PgPool,
    broadcaster: Broadcaster,
    team_id: Uuid,
    manager_id: Uuid,
    target_user_id: Uuid,
) -> Result<(), AppError> {
    check_moderation_target(pool, team_id, manager_id, target_user_id).await?;

    let updated = repo::teams::deactivate_member(pool, team_id, target_user_id).await?;
    if !updated {
        return Err(AppError::Internal("member vanished during kick".into()));
    }

    let event = WsEvent::MemberKicked {
        team_id,
        user_id: target_user_id,
        by: manager_id,
    };

    broadcaster.to_team(team_id, event.clone()).await;
    broadcaster.to_user(target_user_id, event);

    audit::record(
        pool,
        team_id,
        manager_id,
        "member_kicked",
        "team_member",
        target_user_id,
        json!({}),
    )
    .await;

    Ok(())
}

pub async fn ban_member(
    pool: &PgPool,
    broadcaster: Broadcaster,
    team_id: Uuid,
    manager_id: Uuid,
    target_user_id: Uuid,
    expires_at: Option<DateTime<Utc>>,
    reason: Option<String>,
) -> Result<(), AppError> {
    if let Some(exp) = expires_at
        && exp <= Utc::now()
    {
        return Err(AppError::Validation(
            "ban expiry must be in the future".into(),
        ));
    }

    check_moderation_target(pool, team_id, manager_id, target_user_id).await?;

    let mut tx = pool.begin().await?;

    let updated = repo::teams::deactivate_member_tx(&mut tx, team_id, target_user_id).await?;
    if !updated {
        return Err(AppError::Internal("member vanished during ban".into()));
    }

    repo::teams::insert_ban(
        &mut tx,
        Uuid::new_v4(),
        team_id,
        target_user_id,
        manager_id,
        reason.as_deref(),
        expires_at,
    )
    .await?;

    tx.commit().await?;

    let event = WsEvent::MemberBanned {
        team_id,
        user_id: target_user_id,
        expires_at: expires_at.map(|t| t.timestamp()),
        by: manager_id,
    };

    broadcaster.to_team(team_id, event.clone()).await;
    broadcaster.to_user(target_user_id, event);

    audit::record(
        pool,
        team_id,
        manager_id,
        "member_banned",
        "team_member",
        target_user_id,
        json!({
            "expires_at": expires_at.map(|t| t.timestamp()),
            "reason": reason,
        }),
    )
    .await;

    Ok(())
}

pub async fn unban_member(
    pool: &PgPool,
    team_id: Uuid,
    manager_id: Uuid,
    target_user_id: Uuid,
) -> Result<(), AppError> {
    let lifted = repo::teams::lift_active_ban(pool, team_id, target_user_id).await?;
    if !lifted {
        return Err(AppError::NotFound("no active ban found".into()));
    }

    audit::record(
        pool,
        team_id,
        manager_id,
        "member_unbanned",
        "team_member",
        target_user_id,
        json!({}),
    )
    .await;

    Ok(())
}
