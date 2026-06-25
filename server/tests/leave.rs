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
async fn observer_can_leave() {
    let app = spawn_app().await;
    let (alice_token, _) = register_and_login(&app.address, "alice@example.com").await;
    let (bob_token, _) = register_and_login(&app.address, "bob@example.com").await;
    let client = reqwest::Client::new();

    let (team_id, code) = create_team_and_invite(&app.address, &alice_token).await;
    join_team(&app.address, &bob_token, &code).await;

    let res = client
        .post(format!("{}/teams/{team_id}/leave", app.address))
        .bearer_auth(&bob_token)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    let res = client
        .get(format!("{}/teams/{team_id}", app.address))
        .bearer_auth(&bob_token)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn responder_can_leave() {
    let app = spawn_app().await;
    let (alice_token, _) = register_and_login(&app.address, "alice@example.com").await;
    let (bob_token, bob_id) = register_and_login(&app.address, "bob@example.com").await;
    let client = reqwest::Client::new();

    let (team_id, code) = create_team_and_invite(&app.address, &alice_token).await;
    join_team(&app.address, &bob_token, &code).await;

    client
        .patch(format!(
            "{}/teams/{team_id}/members/{bob_id}/role",
            app.address
        ))
        .bearer_auth(&alice_token)
        .json(&json!({ "role": "responder" }))
        .send()
        .await
        .unwrap();

    let res = client
        .post(format!("{}/teams/{team_id}/leave", app.address))
        .bearer_auth(&bob_token)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn manager_cannot_leave_without_transfer() {
    let app = spawn_app().await;
    let (alice_token, _) = register_and_login(&app.address, "alice@example.com").await;
    let client = reqwest::Client::new();

    let (team_id, _) = create_team_and_invite(&app.address, &alice_token).await;

    let res = client
        .post(format!("{}/teams/{team_id}/leave", app.address))
        .bearer_auth(&alice_token)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn left_member_can_rejoin_with_new_code() {
    let app = spawn_app().await;
    let (alice_token, _) = register_and_login(&app.address, "alice@example.com").await;
    let (bob_token, _) = register_and_login(&app.address, "bob@example.com").await;
    let client = reqwest::Client::new();

    let (team_id, code) = create_team_and_invite(&app.address, &alice_token).await;
    join_team(&app.address, &bob_token, &code).await;

    client
        .post(format!("{}/teams/{team_id}/leave", app.address))
        .bearer_auth(&bob_token)
        .send()
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
