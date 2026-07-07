use server::config::Config;
use server::routes;
use server::state::AppState;
use server::ws::{broadcaster::Broadcaster, presence::PresenceTracker};
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

use axum::http::{HeaderValue, Method};
use server::hooks::ReactionRegistry;
use server::hooks::reactions::{
    VigilBlockRelease, VigilCreateIncident, VigilEscalateIncident, VigilValidateReleaseStep,
};
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("server=debug,tower_http=debug")),
        )
        .init();

    let config = Config::from_env()?;

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;

    tracing::info!("Connected to PostgreSQL");

    let registry = ReactionRegistry::builder()
        .register(Arc::new(VigilCreateIncident::new()))
        .register(Arc::new(VigilEscalateIncident::new()))
        .register(Arc::new(VigilBlockRelease::new()))
        .register(Arc::new(VigilValidateReleaseStep::new()))
        .build();

    let state = AppState {
        pool: pool.clone(),
        broadcaster: Broadcaster::new(pool.clone()),
        presence: PresenceTracker::new(),
        webhook_secret: config.webhook_secret,
        master_key: config.master_key,
        registry,
    };

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
        ]);

    let app = routes::router()
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state);

    let addr = format!("{}:{}", config.server_host, config.server_port);
    let listener = TcpListener::bind(&addr).await?;
    tracing::info!("VIGIL server listening on {addr}");

    axum::serve(listener, app).await?;

    Ok(())
}
