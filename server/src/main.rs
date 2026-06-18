use server::config::Config;
use server::routes;
use server::state::AppState;
use server::ws::broadcaster::Broadcaster;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

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

    let broadcaster = Broadcaster::default();
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
