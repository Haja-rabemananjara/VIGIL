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

async fn create_invitation(address: &str, token: &str, team_id: Uuid) -> String {
    let client = reqwest::Client::new();
    let res = client
        .post(format!("{address}/teams/{team_id}/invitations"))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let body: serde_json::Value = res.json().await.unwrap();
    body["code"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn manager_can_create_invitation() {
    let app = spawn_app().await;
    let token = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &token, "Invite Team").await;

    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/teams/{team_id}/invitations", app.address))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["code"].as_str().unwrap().len(), 8);
    assert!(body["expires_at"].as_str().is_some());
}

#[tokio::test]
async fn user_can_join_team_with_valid_code() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let bob = register_and_login(&app.address, "bob@example.com").await;
    let client = reqwest::Client::new();

    let team_id = create_team(&app.address, &alice, "Join Team").await;
    let code = create_invitation(&app.address, &alice, team_id).await;

    let res = client
        .post(format!("{}/teams/join", app.address))
        .bearer_auth(&bob)
        .json(&json!({ "code": code }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["role"], "observer");

    let res = client
        .get(format!("{}/teams/{team_id}", app.address))
        .bearer_auth(&bob)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn invalid_code_returns_404() {
    let app = spawn_app().await;
    let token = register_and_login(&app.address, "alice@example.com").await;
    let client = reqwest::Client::new();

    let res = client
        .post(format!("{}/teams/join", app.address))
        .bearer_auth(&token)
        .json(&json!({ "code": "ZZZZZZZZ" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn already_member_returns_409() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let bob = register_and_login(&app.address, "bob@example.com").await;
    let client = reqwest::Client::new();

    let team_id = create_team(&app.address, &alice, "Dup Team").await;
    let code = create_invitation(&app.address, &alice, team_id).await;

    client
        .post(format!("{}/teams/join", app.address))
        .bearer_auth(&bob)
        .json(&json!({ "code": code }))
        .send()
        .await
        .unwrap();

    let res = client
        .post(format!("{}/teams/join", app.address))
        .bearer_auth(&bob)
        .json(&json!({ "code": code }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn banned_user_cannot_join_returns_403() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let bob = register_and_login(&app.address, "bob@example.com").await;
    let client = reqwest::Client::new();

    let team_id = create_team(&app.address, &alice, "Ban Team").await;
    let code = create_invitation(&app.address, &alice, team_id).await;

    let res = client
        .get(format!("{}/me", app.address))
        .bearer_auth(&bob)
        .send()
        .await
        .unwrap();
    let bob_id: Uuid = {
        let body: serde_json::Value = res.json().await.unwrap();
        Uuid::parse_str(body["id"].as_str().unwrap()).unwrap()
    };

    sqlx::query!(
        r#"
        INSERT INTO team_bans (id, team_id, user_id, created_by, reason)
        VALUES ($1, $2, $3, $4, 'test ban')
        "#,
        Uuid::new_v4(),
        team_id,
        bob_id,
        bob_id,
    )
    .execute(&app.pool)
    .await
    .unwrap();

    let res = client
        .post(format!("{}/teams/join", app.address))
        .bearer_auth(&bob)
        .json(&json!({ "code": code }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn non_manager_cannot_create_invitation() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let bob = register_and_login(&app.address, "bob@example.com").await;
    let client = reqwest::Client::new();

    let team_id = create_team(&app.address, &alice, "Perm Team").await;
    let code = create_invitation(&app.address, &alice, team_id).await;

    client
        .post(format!("{}/teams/join", app.address))
        .bearer_auth(&bob)
        .json(&json!({ "code": code }))
        .send()
        .await
        .unwrap();

    let res = client
        .post(format!("{}/teams/{team_id}/invitations", app.address))
        .bearer_auth(&bob)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}
