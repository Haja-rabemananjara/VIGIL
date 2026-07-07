use reqwest::StatusCode;
use serde_json::json;

mod common;
use common::spawn_app;

async fn register_and_login(address: &str, email: &str) -> String {
    let client = reqwest::Client::new();

    client
        .post(format!("{address}/auth/signup"))
        .json(&json!({ "email": email, "password": "password123", "display_name": "Test" }))
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

#[tokio::test]
async fn create_team_returns_201_and_creator_is_manager() {
    let app = spawn_app().await;
    let token = register_and_login(&app.address, "alice@example.com").await;
    let client = reqwest::Client::new();

    let res = client
        .post(format!("{}/teams", app.address))
        .bearer_auth(&token)
        .json(&json!({ "name": "Platform Squad" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["name"], "Platform Squad");
    assert_eq!(body["role"], "manager");
}

#[tokio::test]
async fn list_teams_returns_only_my_teams() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let bob = register_and_login(&app.address, "bob@example.com").await;
    let client = reqwest::Client::new();

    client
        .post(format!("{}/teams", app.address))
        .bearer_auth(&alice)
        .json(&json!({ "name": "Alice Team" }))
        .send()
        .await
        .unwrap();

    let bob_teams: serde_json::Value = client
        .get(format!("{}/teams", app.address))
        .bearer_auth(&bob)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(bob_teams.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn get_team_as_non_member_returns_404() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let bob = register_and_login(&app.address, "bob@example.com").await;
    let client = reqwest::Client::new();

    let created: serde_json::Value = client
        .post(format!("{}/teams", app.address))
        .bearer_auth(&alice)
        .json(&json!({ "name": "Secret" }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let team_id = created["id"].as_str().unwrap();

    let res = client
        .get(format!("{}/teams/{team_id}", app.address))
        .bearer_auth(&bob)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn create_team_with_blank_name_returns_422() {
    let app = spawn_app().await;
    let token = register_and_login(&app.address, "alice@example.com").await;
    let client = reqwest::Client::new();

    let res = client
        .post(format!("{}/teams", app.address))
        .bearer_auth(&token)
        .json(&json!({ "name": "   " }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn teams_require_authentication() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let res = client
        .get(format!("{}/teams", app.address))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}
