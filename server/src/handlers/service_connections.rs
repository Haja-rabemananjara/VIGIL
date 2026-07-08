use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

use serde::Deserialize;

use crate::domain::service_connections::ServiceConnection;
use crate::error::AppError;
use crate::handlers::auth::AuthUser;
use crate::services;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ConnectServiceInput {
    pub token: String,
}

pub async fn connect(
    State(state): State<AppState>,
    user: AuthUser,
    Path(service_str): Path<String>,
    Json(input): Json<ConnectServiceInput>,
) -> Result<Json<ServiceConnection>, AppError> {
    let service = services::service_connections::parse_service_name(&service_str)?;
    let connection = services::service_connections::connect_service(
        &state.pool,
        &state.master_key,
        user.id,
        service,
        &input.token,
    )
    .await?;
    Ok(Json(connection))
}

pub async fn list(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<ServiceConnection>>, AppError> {
    let connections = services::service_connections::list_connections(&state.pool, user.id).await?;
    Ok(Json(connections))
}

pub async fn disconnect(
    State(state): State<AppState>,
    user: AuthUser,
    Path(service_str): Path<String>,
) -> Result<StatusCode, AppError> {
    let service = services::service_connections::parse_service_name(&service_str)?;
    services::service_connections::disconnect_service(&state.pool, user.id, service).await?;
    Ok(StatusCode::NO_CONTENT)
}
