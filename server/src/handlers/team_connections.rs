use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::team_connections::TeamServiceConnection;
use crate::error::AppError;
use crate::extractors::RequireManager;
use crate::services;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ConnectInput {
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct ConnectResponse {
    pub connection: TeamServiceConnection,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook_url: Option<String>,
}

pub async fn list(
    State(state): State<AppState>,
    RequireManager(member): RequireManager,
    Path(team_id): Path<Uuid>,
) -> Result<Json<Vec<TeamServiceConnection>>, AppError> {
    let _ = (team_id, member); // team_id already validated by RequireManager
    let connections = services::team_connections::list_connections(&state.pool, team_id).await?;
    Ok(Json(connections))
}

pub async fn connect(
    State(state): State<AppState>,
    RequireManager(member): RequireManager,
    Path((team_id, service)): Path<(Uuid, String)>,
    Json(input): Json<ConnectInput>,
) -> Result<(StatusCode, Json<ConnectResponse>), AppError> {
    let connection = services::team_connections::connect_service(
        &state.pool,
        &state.master_key,
        team_id,
        &service,
        &input.token,
        member.user_id,
    )
    .await?;

    // Generate webhook URL only for services that receive incoming webhooks
    let webhook_url = match service.as_str() {
        "github" => {
            let host = std::env::var("PUBLIC_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string());
            Some(services::team_connections::webhook_url(
                connection.id,
                &host,
            ))
        }
        _ => None,
    };

    Ok((
        StatusCode::CREATED,
        Json(ConnectResponse {
            connection,
            webhook_url,
        }),
    ))
}

pub async fn disconnect(
    State(state): State<AppState>,
    RequireManager(member): RequireManager,
    Path((team_id, service)): Path<(Uuid, String)>,
) -> Result<StatusCode, AppError> {
    let _ = member;
    services::team_connections::disconnect_service(&state.pool, team_id, &service).await?;
    Ok(StatusCode::NO_CONTENT)
}
