use axum::{
    extract::{
        Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::Response,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::HashSet;

use crate::{
    state::AppState,
    ws::events::{WsClientMessage, WsEvent},
    ws::presence::ResourceKey,
};

#[derive(Deserialize)]
pub struct WsQuery {
    pub token: String,
}

pub async fn ws_handler(
    State(state): State<AppState>,
    Query(query): Query<WsQuery>,
    ws: WebSocketUpgrade,
) -> Response {
    let raw_token = match hex::decode(&query.token) {
        Ok(bytes) => bytes,
        Err(_) => {
            return axum::http::Response::builder()
                .status(401)
                .body(axum::body::Body::from("invalid token"))
                .unwrap();
        }
    };
    let token_hash = Sha256::digest(&raw_token).to_vec();

    let user_id = match sqlx::query!(
        r#"
        SELECT user_id
        FROM sessions
        WHERE token_hash = $1 AND expires_at > now()
        "#,
        token_hash,
    )
    .fetch_optional(&state.pool)
    .await
    {
        Ok(Some(row)) => row.user_id,
        Ok(None) => {
            return axum::http::Response::builder()
                .status(401)
                .body(axum::body::Body::from("invalid token"))
                .unwrap();
        }
        Err(e) => {
            tracing::error!("ws auth db error: {e}");
            return axum::http::Response::builder()
                .status(500)
                .body(axum::body::Body::from("internal error"))
                .unwrap();
        }
    };

    ws.on_upgrade(move |socket| handle_socket(socket, state, user_id))
}

async fn handle_socket(mut socket: WebSocket, state: AppState, user_id: uuid::Uuid) {
    let mut rx = state.broadcaster.register(user_id);

    let mut watching: HashSet<ResourceKey> = HashSet::new();

    let hello = WsEvent::Connected { user_id };
    if let Ok(json) = serde_json::to_string(&hello) {
        let _ = socket.send(Message::Text(json.into())).await;
    }

    loop {
        tokio::select! {
            event = rx.recv() => {
                match event {
                    Some(event) => {
                        let json = match serde_json::to_string(&event) {
                            Ok(s) => s,
                            Err(e) => {
                                tracing::error!("ws serialize error: {e}");
                                continue;
                            }
                        };
                        if socket.send(Message::Text(json.into())).await.is_err() {
                            break;
                        }
                    }
                    None => break,
                }
            }

            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(cmd) = serde_json::from_str::<WsClientMessage>(&text) {
                            match cmd {
                                WsClientMessage::Watch { resource_type, resource_id, team_id } => {
                                    let key = ResourceKey {
                                        resource_type: resource_type.clone(),
                                        resource_id,
                                    };
                                    watching.insert(key);

                                    let watchers = state.presence.watch(
                                        user_id,
                                        resource_type.clone(),
                                        resource_id,
                                    );

                                    state.broadcaster.to_team(team_id, WsEvent::PresenceUpdate {
                                        team_id,
                                        resource_type,
                                        resource_id,
                                        watchers,
                                    }).await;
                                }
                                WsClientMessage::Unwatch { resource_type, resource_id, team_id } => {
                                    let key = ResourceKey {
                                        resource_type: resource_type.clone(),
                                        resource_id,
                                    };
                                    watching.remove(&key);

                                    let watchers = state.presence.unwatch(
                                        user_id,
                                        resource_type.clone(),
                                        resource_id,
                                    );

                                    state.broadcaster.to_team(team_id, WsEvent::PresenceUpdate {
                                        team_id,
                                        resource_type,
                                        resource_id,
                                        watchers,
                                    }).await;
                                }
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Err(_)) => break,
                    _ => continue,
                }
            }
        }
    }

    let affected = state.presence.disconnect(user_id);
    for (resource_type, resource_id, watchers) in affected {
        if resource_type == "incident"
            && let Ok(Some(row)) =
                sqlx::query!("SELECT team_id FROM incidents WHERE id = $1", resource_id,)
                    .fetch_optional(&state.pool)
                    .await
        {
            state
                .broadcaster
                .to_team(
                    row.team_id,
                    WsEvent::PresenceUpdate {
                        team_id: row.team_id,
                        resource_type: resource_type.clone(),
                        resource_id,
                        watchers,
                    },
                )
                .await;
        }
    }
}
