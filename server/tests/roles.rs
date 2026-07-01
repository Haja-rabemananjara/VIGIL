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

async fn create_team_and_invite(address: &str, manager_token: &str) -> (Uuid, String) {
    let client = reqwest::Client::new();

    let res = client
        .post(format!("{address}/teams"))
        .bearer_auth(manager_token)
        .json(&json!({ "name": "Role Team" }))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = res.json().await.unwrap();
    let team_id = Uuid::parse_str(body["id"].as_str().unwrap()).unwrap();

    let res = client
        .post(format!("{address}/teams/{team_id}/invitations"))
        .bearer_auth(manager_token)
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
async fn manager_can_promote_observer_to_responder() {
    let app = spawn_app().await;
    let (alice_token, _) = register_and_login(&app.address, "alice@example.com").await;
    let (bob_token, bob_id) = register_and_login(&app.address, "bob@example.com").await;
    let client = reqwest::Client::new();

    let (team_id, code) = create_team_and_invite(&app.address, &alice_token).await;
    join_team(&app.address, &bob_token, &code).await;

    let res = client
        .patch(format!(
            "{}/teams/{team_id}/members/{bob_id}/role",
            app.address
        ))
        .bearer_auth(&alice_token)
        .json(&json!({ "role": "responder" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    let res = client
        .get(format!("{}/teams/{team_id}/members", app.address))
        .bearer_auth(&alice_token)
        .send()
        .await
        .unwrap();
    let members: serde_json::Value = res.json().await.unwrap();
    let bob = members
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["user_id"] == bob_id.to_string())
        .unwrap();
    assert_eq!(bob["role"], "responder");
}

#[tokio::test]
async fn manager_can_demote_responder_to_observer() {
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
        .patch(format!(
            "{}/teams/{team_id}/members/{bob_id}/role",
            app.address
        ))
        .bearer_auth(&alice_token)
        .json(&json!({ "role": "observer" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn cannot_promote_to_manager() {
    let app = spawn_app().await;
    let (alice_token, _) = register_and_login(&app.address, "alice@example.com").await;
    let (bob_token, bob_id) = register_and_login(&app.address, "bob@example.com").await;
    let client = reqwest::Client::new();

    let (team_id, code) = create_team_and_invite(&app.address, &alice_token).await;
    join_team(&app.address, &bob_token, &code).await;

    let res = client
        .patch(format!(
            "{}/teams/{team_id}/members/{bob_id}/role",
            app.address
        ))
        .bearer_auth(&alice_token)
        .json(&json!({ "role": "manager" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn cannot_change_own_role() {
    let app = spawn_app().await;
    let (alice_token, alice_id) = register_and_login(&app.address, "alice@example.com").await;
    let client = reqwest::Client::new();

    let (team_id, _) = create_team_and_invite(&app.address, &alice_token).await;

    let res = client
        .patch(format!(
            "{}/teams/{team_id}/members/{alice_id}/role",
            app.address
        ))
        .bearer_auth(&alice_token)
        .json(&json!({ "role": "observer" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn observer_cannot_change_roles() {
    let app = spawn_app().await;
    let (alice_token, alice_id) = register_and_login(&app.address, "alice@example.com").await;
    let (bob_token, _) = register_and_login(&app.address, "bob@example.com").await;
    let client = reqwest::Client::new();

    let (team_id, code) = create_team_and_invite(&app.address, &alice_token).await;
    join_team(&app.address, &bob_token, &code).await;

    let res = client
        .patch(format!(
            "{}/teams/{team_id}/members/{alice_id}/role",
            app.address
        ))
        .bearer_auth(&bob_token)
        .json(&json!({ "role": "observer" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn member_list_visible_to_all_members() {
    let app = spawn_app().await;
    let (alice_token, _) = register_and_login(&app.address, "alice@example.com").await;
    let (bob_token, _) = register_and_login(&app.address, "bob@example.com").await;
    let client = reqwest::Client::new();

    let (team_id, code) = create_team_and_invite(&app.address, &alice_token).await;
    join_team(&app.address, &bob_token, &code).await;

    let res = client
        .get(format!("{}/teams/{team_id}/members", app.address))
        .bearer_auth(&bob_token)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let members: Vec<serde_json::Value> = res.json().await.unwrap();
    assert_eq!(members.len(), 2);
}
