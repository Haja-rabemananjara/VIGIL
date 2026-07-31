use std::collections::HashMap;

use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::AppError,
    repo,
    ws::{Broadcaster, WsEvent},
};

pub const AVAILABLE_EMOJIS: &[&str] = &["+1", "-1", "eyes", "warning", "check", "fire"];

pub fn get_available() -> &'static [&'static str] {
    AVAILABLE_EMOJIS
}

#[derive(Debug, Serialize)]
pub struct IncidentReactions {
    pub reactions: HashMap<Uuid, HashMap<String, Vec<Uuid>>>,
}

pub async fn get_reactions_for_incident(
    pool: &PgPool,
    incident_id: Uuid,
    team_id: Uuid,
) -> Result<IncidentReactions, AppError> {
    let incident = repo::incidents::find_incident(pool, incident_id)
        .await?
        .ok_or_else(|| AppError::NotFound("incident not found".into()))?;

    if incident.team_id != team_id {
        return Err(AppError::NotFound("incident not found".into()));
    }

    let rows = repo::reactions::list_reactions_for_incident(pool, incident_id).await?;

    let mut reactions: HashMap<Uuid, HashMap<String, Vec<Uuid>>> = HashMap::new();
    for row in rows {
        reactions
            .entry(row.entry_id)
            .or_default()
            .entry(row.emoji)
            .or_default()
            .push(row.user_id);
    }

    Ok(IncidentReactions { reactions })
}

pub async fn add_reaction(
    pool: &PgPool,
    broadcaster: Broadcaster,
    entry_id: Uuid,
    user_id: Uuid,
    emoji: String,
) -> Result<(), AppError> {
    if !AVAILABLE_EMOJIS.contains(&emoji.as_str()) {
        return Err(AppError::Validation(format!("unknown emoji: {emoji}")));
    }

    let entry = repo::incidents::find_timeline_entry(pool, entry_id)
        .await?
        .ok_or_else(|| AppError::NotFound("timeline entry not found".into()))?;

    let id = Uuid::new_v4();
    repo::reactions::insert_reaction(pool, id, entry_id, user_id, &emoji).await?;

    let incident = repo::incidents::find_incident(pool, entry.incident_id)
        .await?
        .ok_or_else(|| AppError::Internal("parent incident not found".into()))?;

    broadcaster
        .to_team(
            incident.team_id,
            WsEvent::ReactionAdded {
                team_id: incident.team_id,
                incident_id: entry.incident_id,
                entry_id,
                emoji,
                user_id,
            },
        )
        .await;

    Ok(())
}

pub async fn remove_reaction(
    pool: &PgPool,
    broadcaster: Broadcaster,
    entry_id: Uuid,
    user_id: Uuid,
    emoji: String,
) -> Result<(), AppError> {
    let entry = repo::incidents::find_timeline_entry(pool, entry_id)
        .await?
        .ok_or_else(|| AppError::NotFound("timeline entry not found".into()))?;

    let deleted = repo::reactions::delete_reaction(pool, entry_id, user_id, &emoji).await?;

    if !deleted {
        return Err(AppError::NotFound("reaction not found".into()));
    }

    let incident = repo::incidents::find_incident(pool, entry.incident_id)
        .await?
        .ok_or_else(|| AppError::Internal("parent incident not found".into()))?;

    broadcaster
        .to_team(
            incident.team_id,
            WsEvent::ReactionRemoved {
                team_id: incident.team_id,
                incident_id: entry.incident_id,
                entry_id,
                emoji,
                user_id,
            },
        )
        .await;

    Ok(())
}
