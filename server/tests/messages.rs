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
    let body: serde_json::Value = res.json().await.unwrap();
    body["code"].as_str().unwrap().to_string()
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

#[tokio::test]
async fn send_message_happy_path() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let bob = register_and_login(&app.address, "bob@example.com").await;
    let client = reqwest::Client::new();

    let team_id = create_team(&app.address, &alice, "Ops").await;
    let code = create_invitation(&app.address, &alice, team_id).await;
    join_team(&app.address, &bob, &code).await;

    let bob_id = get_user_id(&app.address, &bob).await;

    let res = client
        .post(format!("{}/messages/{bob_id}", app.address))
        .bearer_auth(&alice)
        .json(&json!({ "content": "Hello Bob!" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["content"], "Hello Bob!");
    assert!(body["id"].as_str().is_some());
    assert!(body["created_at"].as_i64().is_some());
}

#[tokio::test]
async fn cannot_message_yourself() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let client = reqwest::Client::new();

    let alice_id = get_user_id(&app.address, &alice).await;

    let res = client
        .post(format!("{}/messages/{alice_id}", app.address))
        .bearer_auth(&alice)
        .json(&json!({ "content": "Talking to myself" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn cannot_message_without_shared_team() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let charlie = register_and_login(&app.address, "charlie@example.com").await;
    let client = reqwest::Client::new();

    let charlie_id = get_user_id(&app.address, &charlie).await;

    let res = client
        .post(format!("{}/messages/{charlie_id}", app.address))
        .bearer_auth(&alice)
        .json(&json!({ "content": "Hey stranger" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn empty_content_returns_422() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let bob = register_and_login(&app.address, "bob@example.com").await;
    let client = reqwest::Client::new();

    let team_id = create_team(&app.address, &alice, "Ops").await;
    let code = create_invitation(&app.address, &alice, team_id).await;
    join_team(&app.address, &bob, &code).await;

    let bob_id = get_user_id(&app.address, &bob).await;

    let res = client
        .post(format!("{}/messages/{bob_id}", app.address))
        .bearer_auth(&alice)
        .json(&json!({ "content": "   " }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn too_long_content_returns_422() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let bob = register_and_login(&app.address, "bob@example.com").await;
    let client = reqwest::Client::new();

    let team_id = create_team(&app.address, &alice, "Ops").await;
    let code = create_invitation(&app.address, &alice, team_id).await;
    join_team(&app.address, &bob, &code).await;

    let bob_id = get_user_id(&app.address, &bob).await;
    let long_content = "x".repeat(2001);

    let res = client
        .post(format!("{}/messages/{bob_id}", app.address))
        .bearer_auth(&alice)
        .json(&json!({ "content": long_content }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn conversation_is_bilateral() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let bob = register_and_login(&app.address, "bob@example.com").await;
    let client = reqwest::Client::new();

    let team_id = create_team(&app.address, &alice, "Ops").await;
    let code = create_invitation(&app.address, &alice, team_id).await;
    join_team(&app.address, &bob, &code).await;

    let alice_id = get_user_id(&app.address, &alice).await;
    let bob_id = get_user_id(&app.address, &bob).await;

    client
        .post(format!("{}/messages/{bob_id}", app.address))
        .bearer_auth(&alice)
        .json(&json!({ "content": "Hey Bob" }))
        .send()
        .await
        .unwrap();

    client
        .post(format!("{}/messages/{alice_id}", app.address))
        .bearer_auth(&bob)
        .json(&json!({ "content": "Hey Alice" }))
        .send()
        .await
        .unwrap();

    let res = client
        .get(format!("{}/messages/{bob_id}", app.address))
        .bearer_auth(&alice)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body: serde_json::Value = res.json().await.unwrap();
    let messages = body["messages"].as_array().unwrap();
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0]["content"], "Hey Bob");
    assert_eq!(messages[1]["content"], "Hey Alice");

    let res = client
        .get(format!("{}/messages/{alice_id}", app.address))
        .bearer_auth(&bob)
        .send()
        .await
        .unwrap();

    let body: serde_json::Value = res.json().await.unwrap();
    let messages = body["messages"].as_array().unwrap();
    assert_eq!(messages.len(), 2);
}

#[tokio::test]
async fn third_party_cannot_read_conversation() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let bob = register_and_login(&app.address, "bob@example.com").await;
    let charlie = register_and_login(&app.address, "charlie@example.com").await;
    let client = reqwest::Client::new();

    let team_id = create_team(&app.address, &alice, "Ops").await;
    let code = create_invitation(&app.address, &alice, team_id).await;
    join_team(&app.address, &bob, &code).await;

    let bob_id = get_user_id(&app.address, &bob).await;

    client
        .post(format!("{}/messages/{bob_id}", app.address))
        .bearer_auth(&alice)
        .json(&json!({ "content": "Secret stuff" }))
        .send()
        .await
        .unwrap();

    let res = client
        .get(format!("{}/messages/{bob_id}", app.address))
        .bearer_auth(&charlie)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn kicked_member_cannot_message() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let bob = register_and_login(&app.address, "bob@example.com").await;
    let client = reqwest::Client::new();

    let team_id = create_team(&app.address, &alice, "Ops").await;
    let code = create_invitation(&app.address, &alice, team_id).await;
    join_team(&app.address, &bob, &code).await;

    let alice_id = get_user_id(&app.address, &alice).await;
    let bob_id = get_user_id(&app.address, &bob).await;

    client
        .post(format!(
            "{}/teams/{team_id}/members/{bob_id}/kick",
            app.address
        ))
        .bearer_auth(&alice)
        .send()
        .await
        .unwrap();

    let res = client
        .post(format!("{}/messages/{alice_id}", app.address))
        .bearer_auth(&bob)
        .json(&json!({ "content": "Please let me back" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}
