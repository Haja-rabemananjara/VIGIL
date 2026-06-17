use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

mod error;
mod config;
mod ws;
mod state;
mod handlers;
mod routes;
mod repo;
mod domain;
mod services;

use config::Config;
use state::AppState;
use ws::broadcaster::Broadcaster;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("vigil_server=debug,tower_http=debug")),
        )
        .init();

    let config = Config::from_env()?;

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;

    tracing::info!("Connected to PostgreSQL");

    let broadcaster = Broadcaster::new();

    let state = AppState { pool, broadcaster };

    let app = routes::router()
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(state);

    let addr = format!("{}:{}", config.server_host, config.server_port);
    let listener = TcpListener::bind(&addr).await?;
    tracing::info!("VIGIL server listening on {addr}");

    axum::serve(listener, app).await?;

    Ok(())
}
