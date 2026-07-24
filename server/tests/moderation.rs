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
        .json(&json!({ "name": "Leave Team" }))
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
async fn manager_can_kick_observer() {
    let app = spawn_app().await;
    let (alice_token, _) = register_and_login(&app.address, "alice@example.com").await;
    let (bob_token, bob_id) = register_and_login(&app.address, "bob@example.com").await;
    let client = reqwest::Client::new();

    let (team_id, code) = create_team_and_invite(&app.address, &alice_token).await;
    join_team(&app.address, &bob_token, &code).await;

    // Kick Bob
    let res = client
        .post(format!(
            "{}/teams/{team_id}/members/{bob_id}/kick",
            app.address
        ))
        .bearer_auth(&alice_token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    // Bob no longer has access
    let res = client
        .get(format!("{}/teams/{team_id}", app.address))
        .bearer_auth(&bob_token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    // Bob CAN rejoin with a new code (kick, not ban)
    let res = client
        .post(format!("{}/teams/{team_id}/invitations", app.address))
        .bearer_auth(&alice_token)
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = res.json().await.unwrap();
    let new_code = body["code"].as_str().unwrap();

    let res = client
        .post(format!("{}/teams/join", app.address))
        .bearer_auth(&bob_token)
        .json(&json!({ "code": new_code }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn cannot_kick_yourself() {
    let app = spawn_app().await;
    let (alice_token, alice_id) = register_and_login(&app.address, "alice@example.com").await;
    let client = reqwest::Client::new();

    let (team_id, _) = create_team_and_invite(&app.address, &alice_token).await;

    let res = client
        .post(format!(
            "{}/teams/{team_id}/members/{alice_id}/kick",
            app.address
        ))
        .bearer_auth(&alice_token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn permanent_ban_blocks_join_forever() {
    let app = spawn_app().await;
    let (alice_token, _) = register_and_login(&app.address, "alice@example.com").await;
    let (bob_token, bob_id) = register_and_login(&app.address, "bob@example.com").await;
    let client = reqwest::Client::new();

    let (team_id, code) = create_team_and_invite(&app.address, &alice_token).await;
    join_team(&app.address, &bob_token, &code).await;

    // Permanent ban (no expires_at)
    let res = client
        .post(format!(
            "{}/teams/{team_id}/members/{bob_id}/ban",
            app.address
        ))
        .bearer_auth(&alice_token)
        .json(&json!({ "expires_at": null, "reason": "spam" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    // Bob cannot rejoin
    let res = client
        .post(format!("{}/teams/{team_id}/invitations", app.address))
        .bearer_auth(&alice_token)
        .send()
        .await
        .unwrap();
    let new_code: serde_json::Value = res.json().await.unwrap();

    let res = client
        .post(format!("{}/teams/join", app.address))
        .bearer_auth(&bob_token)
        .json(&json!({ "code": new_code["code"] }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn temporary_ban_expires_naturally() {
    let app = spawn_app().await;
    let (alice_token, _) = register_and_login(&app.address, "alice@example.com").await;
    let (bob_token, bob_id) = register_and_login(&app.address, "bob@example.com").await;
    let client = reqwest::Client::new();

    let (team_id, code) = create_team_and_invite(&app.address, &alice_token).await;
    join_team(&app.address, &bob_token, &code).await;

    let tomorrow = chrono::Utc::now().timestamp() + 86400;
    let res = client
        .post(format!("{}/teams/{team_id}/members/{bob_id}/ban", app.address))
        .bearer_auth(&alice_token)
        .json(&json!({ "expires_at": tomorrow }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    sqlx::query!(
        r#"
        UPDATE team_bans
        SET created_at = now() - interval '2 days',
            expires_at = now() - interval '1 day'
        WHERE team_id = $1 AND user_id = $2 AND status = 'active'
        "#,
        team_id,
        bob_id,
    )
    .execute(&app.pool)
    .await
    .unwrap();

    let res = client
        .post(format!("{}/teams/{team_id}/invitations", app.address))
        .bearer_auth(&alice_token)
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = res.json().await.unwrap();
    let new_code = body["code"].as_str().unwrap();

    let res = client
        .post(format!("{}/teams/join", app.address))
        .bearer_auth(&bob_token)
        .json(&json!({ "code": new_code }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn unban_allows_rejoin() {
    let app = spawn_app().await;
    let (alice_token, _) = register_and_login(&app.address, "alice@example.com").await;
    let (bob_token, bob_id) = register_and_login(&app.address, "bob@example.com").await;
    let client = reqwest::Client::new();

    let (team_id, code) = create_team_and_invite(&app.address, &alice_token).await;
    join_team(&app.address, &bob_token, &code).await;

    // Ban permanently
    client
        .post(format!(
            "{}/teams/{team_id}/members/{bob_id}/ban",
            app.address
        ))
        .bearer_auth(&alice_token)
        .json(&json!({ "expires_at": null }))
        .send()
        .await
        .unwrap();

    // Unban
    let res = client
        .delete(format!("{}/teams/{team_id}/bans/{bob_id}", app.address))
        .bearer_auth(&alice_token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    // Bob can now rejoin
    let res = client
        .post(format!("{}/teams/{team_id}/invitations", app.address))
        .bearer_auth(&alice_token)
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = res.json().await.unwrap();
    let new_code = body["code"].as_str().unwrap();

    let res = client
        .post(format!("{}/teams/join", app.address))
        .bearer_auth(&bob_token)
        .json(&json!({ "code": new_code }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn cannot_ban_the_manager() {
    let app = spawn_app().await;
    let (alice_token, alice_id) = register_and_login(&app.address, "alice@example.com").await;
    let (bob_token, _) = register_and_login(&app.address, "bob@example.com").await;
    let client = reqwest::Client::new();

    let (team_id, code) = create_team_and_invite(&app.address, &alice_token).await;
    join_team(&app.address, &bob_token, &code).await;

    // Even a Manager can't ban another Manager
    let res = client
        .post(format!(
            "{}/teams/{team_id}/members/{alice_id}/ban",
            app.address
        ))
        .bearer_auth(&alice_token)
        .json(&json!({ "expires_at": null }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn ban_with_past_expiry_returns_422() {
    let app = spawn_app().await;
    let (alice_token, _) = register_and_login(&app.address, "alice@example.com").await;
    let (bob_token, bob_id) = register_and_login(&app.address, "bob@example.com").await;
    let client = reqwest::Client::new();

    let (team_id, code) = create_team_and_invite(&app.address, &alice_token).await;
    join_team(&app.address, &bob_token, &code).await;

    let past = chrono::Utc::now().timestamp() - 3600;
    let res = client
        .post(format!("{}/teams/{team_id}/members/{bob_id}/ban", app.address))
        .bearer_auth(&alice_token)
        .json(&json!({ "expires_at": past }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}