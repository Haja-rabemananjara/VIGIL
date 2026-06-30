use server::routes;
use server::state::AppState;
use server::ws::broadcaster::Broadcaster;
use sqlx::{Executor, PgPool};
use tokio::net::TcpListener;
use uuid::Uuid;

#[allow(dead_code)]
pub struct TestApp {
    pub address: String,
    pub pool: PgPool,
    pub db_name: String,
    pub client: reqwest::Client,
}

#[allow(dead_code)]
impl TestApp {
    pub async fn cleanup(self) {
        self.pool.close().await;

        let maintenance_url = maintenance_url();
        let maintenance_pool = PgPool::connect(&maintenance_url).await.unwrap();

        maintenance_pool
            .execute(
                format!(
                    "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{}'",
                    self.db_name
                )
                .as_str(),
            )
            .await
            .unwrap();

        maintenance_pool
            .execute(format!("DROP DATABASE IF EXISTS \"{}\"", self.db_name).as_str())
            .await
            .unwrap();
    }
}

pub async fn spawn_app() -> TestApp {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("server=debug,sqlx=warn")),
        )
        .with_test_writer()
        .try_init();

    let db_name = format!("vigil_test_{}", Uuid::new_v4().simple());

    let maintenance_url = maintenance_url();
    let maintenance_pool = PgPool::connect(&maintenance_url).await.unwrap();
    maintenance_pool
        .execute(format!("CREATE DATABASE \"{}\"", db_name).as_str())
        .await
        .unwrap();

    let db_url = format!("postgres://vigil:vigil_dev@localhost:5432/{}", db_name);
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .unwrap();

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations on test DB");

    let broadcaster = Broadcaster::new(pool.clone());
    let state = AppState {
        pool: pool.clone(),
        broadcaster,
    };

    let app = routes::router()
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    TestApp {
        address: format!("http://127.0.0.1:{}", port),
        pool,
        db_name,
        client,
    }
}

fn maintenance_url() -> String {
    "postgres://vigil:vigil_dev@localhost:5432/postgres".to_string()
}
