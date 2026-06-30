use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::Response,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::{state::AppState, ws::events::WsEvent};

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
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Err(_)) => break,
                    _ => continue,
                }
            }
        }
    }
}
