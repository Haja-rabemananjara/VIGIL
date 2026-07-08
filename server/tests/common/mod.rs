use server::routes;
use server::state::AppState;
use server::ws::PresenceTracker;
use server::ws::broadcaster::Broadcaster;
use sqlx::{Executor, PgPool};
use std::sync::OnceLock;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use uuid::Uuid;

const TEMPLATE_DB_NAME: &str = "vigil_test_template";

static TEMPLATE_INIT: OnceLock<Mutex<bool>> = OnceLock::new();

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

async fn ensure_template_exists() {
    let flag = TEMPLATE_INIT.get_or_init(|| Mutex::new(false));
    let mut initialized = flag.lock().await;

    if *initialized {
        return;
    }

    let maintenance_pool = PgPool::connect(&maintenance_url()).await.unwrap();

    let exists: (bool,) =
        sqlx::query_as("SELECT EXISTS(SELECT 1 FROM pg_database WHERE datname = $1)")
            .bind(TEMPLATE_DB_NAME)
            .fetch_one(&maintenance_pool)
            .await
            .unwrap();

    if exists.0 {
        *initialized = true;
        return;
    }

    maintenance_pool
        .execute(format!("CREATE DATABASE \"{}\"", TEMPLATE_DB_NAME).as_str())
        .await
        .unwrap();

    let template_url = format!(
        "postgres://vigil:vigil_dev@localhost:5432/{}",
        TEMPLATE_DB_NAME
    );
    let template_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&template_url)
        .await
        .unwrap();

    sqlx::migrate!("./migrations")
        .run(&template_pool)
        .await
        .expect("Failed to run migrations on template DB");

    template_pool.close().await;

    *initialized = true;
}

pub async fn spawn_app() -> TestApp {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("server=debug,sqlx=warn")),
        )
        .with_test_writer()
        .try_init();

    ensure_template_exists().await;

    let db_name = format!("vigil_test_{}", Uuid::new_v4().simple());
    let maintenance_pool = PgPool::connect(&maintenance_url()).await.unwrap();
    maintenance_pool
        .execute(
            format!(
                "CREATE DATABASE \"{}\" TEMPLATE \"{}\"",
                db_name, TEMPLATE_DB_NAME
            )
            .as_str(),
        )
        .await
        .unwrap();
    maintenance_pool.close().await;

    let db_url = format!("postgres://vigil:vigil_dev@localhost:5432/{}", db_name);
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .unwrap();

    let broadcaster = Broadcaster::new(pool.clone());
    let presence = PresenceTracker::new();
    let registry = server::hooks::ReactionRegistry::builder()
        .register(std::sync::Arc::new(
            server::hooks::reactions::VigilCreateIncident::new(),
        ))
        .register(std::sync::Arc::new(
            server::hooks::reactions::VigilEscalateIncident::new(),
        ))
        .register(std::sync::Arc::new(
            server::hooks::reactions::VigilBlockRelease::new(),
        ))
        .register(std::sync::Arc::new(
            server::hooks::reactions::VigilValidateReleaseStep::new(),
        ))
        .register(std::sync::Arc::new(
            server::hooks::reactions::DiscordMessage::new(),
        ))
        .build();

    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap();

    let action_catalog = server::hooks::ActionCatalog::builder()
        .register("github", "workflow_run", "A CI workflow run has completed")
        .register("github", "push", "New commits pushed to a branch")
        .register(
            "github",
            "pull_request",
            "A pull request has been opened/updated/closed",
        )
        .build();

    let kickoff_token = server::config::compute_kickoff_token("test_first", "test_login");

    let state = AppState {
        pool: pool.clone(),
        broadcaster,
        presence,
        webhook_secret: "test-webhook-secret".to_string(),
        master_key: [0x42; 32],
        registry,
        http_client,
        action_catalog,
        kickoff_token,
    };

    let app = routes::router()
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap();
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
