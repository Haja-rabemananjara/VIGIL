use reqwest::StatusCode;
use serde_json::json;
use uuid::Uuid;

mod common;
use common::spawn_app;

async fn register_and_login(address: &str, email: &str) -> String {
    let client = reqwest::Client::new();

    client
        .post(format!("{address}/auth/signup"))
        .json(&json!({
            "email": email,
            "password": "password123",
            "display_name": "Test"
        }))
        .send()
        .await
        .unwrap();

    let res = client
        .post(format!("{address}/auth/signin"))
        .json(&json!({ "email": email, "password": "password123" }))
        .send()
        .await
        .unwrap();

    let body: serde_json::Value = res.json().await.unwrap();
    body["token"].as_str().unwrap().to_string()
}

async fn get_user_id(address: &str, token: &str) -> Uuid {
    let client = reqwest::Client::new();

    let res = client
        .get(format!("{address}/me"))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();

    let body: serde_json::Value = res.json().await.unwrap();
    Uuid::parse_str(body["id"].as_str().unwrap()).unwrap()
}

async fn create_team(address: &str, token: &str, name: &str) -> Uuid {
    let client = reqwest::Client::new();

    let res = client
        .post(format!("{address}/teams"))
        .bearer_auth(token)
        .json(&json!({ "name": name }))
        .send()
        .await
        .unwrap();

    let body: serde_json::Value = res.json().await.unwrap();
    Uuid::parse_str(body["id"].as_str().unwrap()).unwrap()
}

async fn add_member_directly(pool: &sqlx::PgPool, team_id: Uuid, user_id: Uuid, role: &str) {
    sqlx::query!(
        r#"
        INSERT INTO team_members (id, team_id, user_id, role)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        team_id,
        user_id,
        role,
    )
    .execute(pool)
    .await
    .unwrap();
}

#[tokio::test]
async fn non_member_gets_404_on_team_routes() {
    let app = spawn_app().await;
    let alice_token = register_and_login(&app.address, "alice@example.com").await;
    let bob_token = register_and_login(&app.address, "bob@example.com").await;
    let client = reqwest::Client::new();

    let team_id = create_team(&app.address, &alice_token, "Secret Team").await;

    let res = client
        .get(format!("{}/teams/{team_id}", app.address))
        .bearer_auth(&bob_token)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn observer_can_read_team() {
    let app = spawn_app().await;
    let alice_token = register_and_login(&app.address, "alice@example.com").await;
    let bob_token = register_and_login(&app.address, "bob@example.com").await;
    let bob_id = get_user_id(&app.address, &bob_token).await;
    let client = reqwest::Client::new();

    let team_id = create_team(&app.address, &alice_token, "Visible Team").await;

    add_member_directly(&app.pool, team_id, bob_id, "observer").await;

    let res = client
        .get(format!("{}/teams/{team_id}", app.address))
        .bearer_auth(&bob_token)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["role"], "observer");
}

#[tokio::test]
async fn responder_can_read_team() {
    let app = spawn_app().await;
    let alice_token = register_and_login(&app.address, "alice@example.com").await;
    let charlie_token = register_and_login(&app.address, "charlie@example.com").await;
    let charlie_id = get_user_id(&app.address, &charlie_token).await;
    let client = reqwest::Client::new();

    let team_id = create_team(&app.address, &alice_token, "Team R").await;

    add_member_directly(&app.pool, team_id, charlie_id, "responder").await;

    let res = client
        .get(format!("{}/teams/{team_id}", app.address))
        .bearer_auth(&charlie_token)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["role"], "responder");
}

#[tokio::test]
async fn manager_can_read_team() {
    let app = spawn_app().await;
    let alice_token = register_and_login(&app.address, "alice@example.com").await;
    let client = reqwest::Client::new();

    let team_id = create_team(&app.address, &alice_token, "Team M").await;

    let res = client
        .get(format!("{}/teams/{team_id}", app.address))
        .bearer_auth(&alice_token)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["role"], "manager");
}

#[tokio::test]
async fn unauthenticated_gets_401_on_team_routes() {
    let app = spawn_app().await;
    let alice_token = register_and_login(&app.address, "alice@example.com").await;
    let client = reqwest::Client::new();

    let team_id = create_team(&app.address, &alice_token, "Auth Team").await;

    let res = client
        .get(format!("{}/teams/{team_id}", app.address))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}
