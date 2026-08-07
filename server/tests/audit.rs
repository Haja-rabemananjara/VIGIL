use reqwest::StatusCode;
use serde_json::json;
use uuid::Uuid;

mod common;
use common::spawn_app;

async fn register_and_login(address: &str, email: &str) -> (String, Uuid) {
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
    let token = body["token"].as_str().unwrap().to_string();

    let res = client
        .get(format!("{address}/me"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    let me: serde_json::Value = res.json().await.unwrap();
    let user_id = Uuid::parse_str(me["id"].as_str().unwrap()).unwrap();

    (token, user_id)
}

async fn create_team_and_invite(address: &str, token: &str) -> (Uuid, String) {
    let client = reqwest::Client::new();

    let res = client
        .post(format!("{address}/teams"))
        .bearer_auth(token)
        .json(&json!({ "name": "Audit Team" }))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = res.json().await.unwrap();
    let team_id = Uuid::parse_str(body["id"].as_str().unwrap()).unwrap();

    let res = client
        .post(format!("{address}/teams/{team_id}/invitations"))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = res.json().await.unwrap();
    let code = body["code"].as_str().unwrap().to_string();

    (team_id, code)
}

async fn join_team(address: &str, token: &str, code: &str) {
    let client = reqwest::Client::new();
    client
        .post(format!("{address}/teams/join"))
        .bearer_auth(token)
        .json(&json!({ "code": code }))
        .send()
        .await
        .unwrap();
}

#[tokio::test]
async fn audit_log_empty_initially() {
    let app = spawn_app().await;
    let (token, _) = register_and_login(&app.address, "alice@example.com").await;
    let (team_id, _) = create_team_and_invite(&app.address, &token).await;

    let res = app
        .client
        .get(format!("{}/teams/{team_id}/audit", app.address))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let entries: Vec<serde_json::Value> = res.json().await.unwrap();
    assert!(entries.is_empty());
}

#[tokio::test]
async fn kick_creates_audit_entry() {
    let app = spawn_app().await;
    let (alice_token, _) = register_and_login(&app.address, "alice@example.com").await;
    let (bob_token, bob_id) = register_and_login(&app.address, "bob@example.com").await;

    let (team_id, code) = create_team_and_invite(&app.address, &alice_token).await;
    join_team(&app.address, &bob_token, &code).await;

    app.client
        .post(format!(
            "{}/teams/{team_id}/members/{bob_id}/kick",
            app.address
        ))
        .bearer_auth(&alice_token)
        .send()
        .await
        .unwrap();

    let res = app
        .client
        .get(format!("{}/teams/{team_id}/audit", app.address))
        .bearer_auth(&alice_token)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let entries: Vec<serde_json::Value> = res.json().await.unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["action"], "member_kicked");
    assert_eq!(entries[0]["entity_id"], bob_id.to_string());
}

#[tokio::test]
async fn ban_creates_audit_entry() {
    let app = spawn_app().await;
    let (alice_token, _) = register_and_login(&app.address, "alice@example.com").await;
    let (bob_token, bob_id) = register_and_login(&app.address, "bob@example.com").await;

    let (team_id, code) = create_team_and_invite(&app.address, &alice_token).await;
    join_team(&app.address, &bob_token, &code).await;

    app.client
        .post(format!(
            "{}/teams/{team_id}/members/{bob_id}/ban",
            app.address
        ))
        .bearer_auth(&alice_token)
        .json(&json!({}))
        .send()
        .await
        .unwrap();

    let res = app
        .client
        .get(format!("{}/teams/{team_id}/audit", app.address))
        .bearer_auth(&alice_token)
        .send()
        .await
        .unwrap();

    let entries: Vec<serde_json::Value> = res.json().await.unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["action"], "member_banned");
}

#[tokio::test]
async fn role_change_creates_audit_entry() {
    let app = spawn_app().await;
    let (alice_token, _) = register_and_login(&app.address, "alice@example.com").await;
    let (bob_token, bob_id) = register_and_login(&app.address, "bob@example.com").await;

    let (team_id, code) = create_team_and_invite(&app.address, &alice_token).await;
    join_team(&app.address, &bob_token, &code).await;

    app.client
        .patch(format!(
            "{}/teams/{team_id}/members/{bob_id}/role",
            app.address
        ))
        .bearer_auth(&alice_token)
        .json(&json!({ "role": "responder" }))
        .send()
        .await
        .unwrap();

    let res = app
        .client
        .get(format!("{}/teams/{team_id}/audit", app.address))
        .bearer_auth(&alice_token)
        .send()
        .await
        .unwrap();

    let entries: Vec<serde_json::Value> = res.json().await.unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["action"], "member_role_responder");
}

#[tokio::test]
async fn observer_cannot_read_audit() {
    let app = spawn_app().await;
    let (alice_token, _) = register_and_login(&app.address, "alice@example.com").await;
    let (bob_token, _) = register_and_login(&app.address, "bob@example.com").await;

    let (team_id, code) = create_team_and_invite(&app.address, &alice_token).await;
    join_team(&app.address, &bob_token, &code).await;

    let res = app
        .client
        .get(format!("{}/teams/{team_id}/audit", app.address))
        .bearer_auth(&bob_token)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn audit_respects_pagination() {
    let app = spawn_app().await;
    let (alice_token, _) = register_and_login(&app.address, "alice@example.com").await;
    let (bob_token, bob_id) = register_and_login(&app.address, "bob@example.com").await;

    let (team_id, code) = create_team_and_invite(&app.address, &alice_token).await;
    join_team(&app.address, &bob_token, &code).await;

    app.client
        .patch(format!(
            "{}/teams/{team_id}/members/{bob_id}/role",
            app.address
        ))
        .bearer_auth(&alice_token)
        .json(&json!({ "role": "responder" }))
        .send()
        .await
        .unwrap();

    app.client
        .patch(format!(
            "{}/teams/{team_id}/members/{bob_id}/role",
            app.address
        ))
        .bearer_auth(&alice_token)
        .json(&json!({ "role": "observer" }))
        .send()
        .await
        .unwrap();

    let res = app
        .client
        .get(format!("{}/teams/{team_id}/audit?limit=1", app.address))
        .bearer_auth(&alice_token)
        .send()
        .await
        .unwrap();

    let entries: Vec<serde_json::Value> = res.json().await.unwrap();
    assert_eq!(entries.len(), 1);

    let res = app
        .client
        .get(format!(
            "{}/teams/{team_id}/audit?limit=1&offset=1",
            app.address
        ))
        .bearer_auth(&alice_token)
        .send()
        .await
        .unwrap();

    let entries: Vec<serde_json::Value> = res.json().await.unwrap();
    assert_eq!(entries.len(), 1);
}
