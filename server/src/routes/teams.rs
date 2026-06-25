use crate::handlers;
use crate::state::AppState;
use axum::routing::patch;
use axum::{
    Router,
    routing::{get, post},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/teams",
            post(handlers::teams::create_team).get(handlers::teams::list_teams),
        )
        .route("/teams/{team_id}", get(handlers::teams::get_team))
        .route(
            "/teams/{team_id}/invitations",
            post(handlers::invitations::create_invitation),
        )
        .route("/teams/join", post(handlers::invitations::join_team))
        .route(
            "/teams/{team_id}/members",
            get(handlers::teams::list_members),
        )
        .route(
            "/teams/{team_id}/members/{user_id}/role",
            patch(handlers::teams::change_member_role),
        )
        .route(
            "/teams/{team_id}/transfer-manager",
            post(handlers::teams::transfer_manager),
        )
}
