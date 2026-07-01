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
        .json(&json!({ "name": "Transfer Team" }))
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

async fn get_member_role(address: &str, token: &str, team_id: Uuid, user_id: Uuid) -> String {
    let client = reqwest::Client::new();
    let res = client
        .get(format!("{address}/teams/{team_id}/members"))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    let members: Vec<serde_json::Value> = res.json().await.unwrap();
    members
        .iter()
        .find(|m| m["user_id"] == user_id.to_string())
        .unwrap()["role"]
        .as_str()
        .unwrap()
        .to_string()
}

#[tokio::test]
async fn transfer_manager_swaps_roles() {
    let app = spawn_app().await;
    let (alice_token, alice_id) = register_and_login(&app.address, "alice@example.com").await;
    let (bob_token, bob_id) = register_and_login(&app.address, "bob@example.com").await;
    let client = reqwest::Client::new();

    let (team_id, code) = create_team_and_invite(&app.address, &alice_token).await;
    join_team(&app.address, &bob_token, &code).await;

    let res = client
        .post(format!("{}/teams/{team_id}/transfer-manager", app.address))
        .bearer_auth(&alice_token)
        .json(&json!({ "target_user_id": bob_id }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    let alice_role = get_member_role(&app.address, &bob_token, team_id, alice_id).await;
    let bob_role = get_member_role(&app.address, &bob_token, team_id, bob_id).await;

    assert_eq!(alice_role, "responder");
    assert_eq!(bob_role, "manager");
}

#[tokio::test]
async fn transfer_to_self_returns_422() {
    let app = spawn_app().await;
    let (alice_token, alice_id) = register_and_login(&app.address, "alice@example.com").await;
    let client = reqwest::Client::new();

    let (team_id, _) = create_team_and_invite(&app.address, &alice_token).await;

    let res = client
        .post(format!("{}/teams/{team_id}/transfer-manager", app.address))
        .bearer_auth(&alice_token)
        .json(&json!({ "target_user_id": alice_id }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn transfer_to_non_member_returns_404() {
    let app = spawn_app().await;
    let (alice_token, _) = register_and_login(&app.address, "alice@example.com").await;
    let (_, bob_id) = register_and_login(&app.address, "bob@example.com").await;
    let client = reqwest::Client::new();

    let (team_id, _) = create_team_and_invite(&app.address, &alice_token).await;

    let res = client
        .post(format!("{}/teams/{team_id}/transfer-manager", app.address))
        .bearer_auth(&alice_token)
        .json(&json!({ "target_user_id": bob_id }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn non_manager_cannot_transfer() {
    let app = spawn_app().await;
    let (alice_token, alice_id) = register_and_login(&app.address, "alice@example.com").await;
    let (bob_token, _) = register_and_login(&app.address, "bob@example.com").await;
    let client = reqwest::Client::new();

    let (team_id, code) = create_team_and_invite(&app.address, &alice_token).await;
    join_team(&app.address, &bob_token, &code).await;

    let res = client
        .post(format!("{}/teams/{team_id}/transfer-manager", app.address))
        .bearer_auth(&bob_token)
        .json(&json!({ "target_user_id": alice_id }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn old_manager_loses_manager_privileges_after_transfer() {
    let app = spawn_app().await;
    let (alice_token, _) = register_and_login(&app.address, "alice@example.com").await;
    let (bob_token, bob_id) = register_and_login(&app.address, "bob@example.com").await;
    let client = reqwest::Client::new();

    let (team_id, code) = create_team_and_invite(&app.address, &alice_token).await;
    join_team(&app.address, &bob_token, &code).await;

    client
        .post(format!("{}/teams/{team_id}/transfer-manager", app.address))
        .bearer_auth(&alice_token)
        .json(&json!({ "target_user_id": bob_id }))
        .send()
        .await
        .unwrap();

    let res = client
        .post(format!("{}/teams/{team_id}/invitations", app.address))
        .bearer_auth(&alice_token)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}
