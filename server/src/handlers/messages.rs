use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{error::AppError, handlers::auth::AuthUser, services, state::AppState};

#[derive(Deserialize)]
pub struct SendMessageBody {
    pub content: String,
}

pub async fn send_message(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(recipient_id): Path<Uuid>,
    Json(body): Json<SendMessageBody>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let msg = services::messages::send_message(
        &state.pool,
        state.broadcaster,
        auth_user.id,
        recipient_id,
        body.content,
    )
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(msg).unwrap()),
    ))
}

#[derive(Deserialize)]
pub struct ConversationQuery {
    pub before: Option<i64>,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    50
}

pub async fn get_conversation(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(other_id): Path<Uuid>,
    Query(params): Query<ConversationQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let messages = services::messages::get_conversation(
        &state.pool,
        auth_user.id,
        other_id,
        params.before,
        params.limit,
    )
    .await?;

    Ok(Json(serde_json::json!({ "messages": messages })))
}
