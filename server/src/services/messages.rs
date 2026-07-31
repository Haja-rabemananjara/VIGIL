use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::AppError,
    repo,
    ws::{Broadcaster, WsEvent},
};

pub const MESSAGE_MAX_LENGTH: usize = 2000;

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub recipient_id: Uuid,
    pub content: String,
    pub created_at: i64,
}

impl MessageResponse {
    fn from_row(row: repo::messages::MessageRow) -> Self {
        Self {
            id: row.id,
            sender_id: row.sender_id,
            recipient_id: row.recipient_id,
            content: row.content,
            created_at: row.created_at.timestamp(),
        }
    }
}

pub async fn send_message(
    pool: &PgPool,
    broadcaster: Broadcaster,
    sender_id: Uuid,
    recipient_id: Uuid,
    content: String,
) -> Result<MessageResponse, AppError> {
    if sender_id == recipient_id {
        return Err(AppError::Validation("cannot message yourself".into()));
    }

    if content.trim().is_empty() {
        return Err(AppError::Validation("content cannot be empty".into()));
    }
    if content.len() > MESSAGE_MAX_LENGTH {
        return Err(AppError::Validation(format!(
            "content exceeds {MESSAGE_MAX_LENGTH} characters"
        )));
    }

    let shared = repo::messages::share_a_team(pool, sender_id, recipient_id).await?;
    if !shared {
        return Err(AppError::Forbidden(
            "you do not share a team with this user".into(),
        ));
    }

    let id = Uuid::new_v4();
    let row = repo::messages::insert_message(pool, id, sender_id, recipient_id, &content).await?;
    let at = row.created_at.timestamp();

    let event = WsEvent::PrivateMessageReceived {
        from: sender_id,
        to: recipient_id,
        message_id: id,
        content: content.clone(),
        at,
    };
    broadcaster.to_user(sender_id, event.clone());
    broadcaster.to_user(recipient_id, event);

    Ok(MessageResponse::from_row(row))
}

pub async fn get_conversation(
    pool: &PgPool,
    caller_id: Uuid,
    other_id: Uuid,
    before_ts: Option<i64>,
    limit: i64,
) -> Result<Vec<MessageResponse>, AppError> {
    let shared = repo::messages::share_a_team(pool, caller_id, other_id).await?;
    if !shared {
        return Err(AppError::Forbidden(
            "you do not share a team with this user".into(),
        ));
    }

    let before = before_ts
        .map(|ts| chrono::DateTime::from_timestamp(ts, 0).unwrap_or_else(chrono::Utc::now));

    let limit = limit.clamp(1, 100);

    let rows = repo::messages::list_conversation(pool, caller_id, other_id, before, limit).await?;

    Ok(rows.into_iter().map(MessageResponse::from_row).collect())
}
