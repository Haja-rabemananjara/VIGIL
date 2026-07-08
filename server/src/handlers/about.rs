use axum::Json;
use axum::extract::{ConnectInfo, State};
use std::net::SocketAddr;

use crate::services::about::{self, AboutResponse};
use crate::state::AppState;

pub async fn get_about(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Json<AboutResponse> {
    let client_host = addr.ip().to_string();

    Json(about::build_response(
        client_host,
        &state.action_catalog,
        &state.registry,
        state.kickoff_token.clone(),
    ))
}
