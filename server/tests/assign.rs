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

async fn create_incident(address: &str, token: &str, team_id: Uuid) -> Uuid {
    let client = reqwest::Client::new();
    let res = client
        .post(format!("{address}/teams/{team_id}/incidents"))
        .bearer_auth(token)
        .json(&json!({ "title": "Test incident", "severity": "low" }))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = res.json().await.unwrap();
    Uuid::parse_str(body["id"].as_str().unwrap()).unwrap()
}

async fn promote_to_responder(address: &str, manager_token: &str, team_id: Uuid, user_id: Uuid) {
    let client = reqwest::Client::new();
    client
        .patch(format!("{address}/teams/{team_id}/members/{user_id}/role"))
        .bearer_auth(manager_token)
        .json(&json!({ "role": "responder" }))
        .send()
        .await
        .unwrap();
}

#[tokio::test]
async fn manager_can_assign_responder() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let bob = register_and_login(&app.address, "bob@example.com").await;
    let client = reqwest::Client::new();

    let team_id = create_team(&app.address, &alice, "Ops").await;
    let code = create_invitation(&app.address, &alice, team_id).await;
    join_team(&app.address, &bob, &code).await;
    let bob_id = get_user_id(&app.address, &bob).await;
    promote_to_responder(&app.address, &alice, team_id, bob_id).await;

    let incident_id = create_incident(&app.address, &alice, team_id).await;

    let res = client
        .post(format!(
            "{}/teams/{team_id}/incidents/{incident_id}/assign",
            app.address
        ))
        .bearer_auth(&alice)
        .json(&json!({ "user_id": bob_id }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn cannot_assign_observer() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let bob = register_and_login(&app.address, "bob@example.com").await;
    let client = reqwest::Client::new();

    let team_id = create_team(&app.address, &alice, "Ops").await;
    let code = create_invitation(&app.address, &alice, team_id).await;
    join_team(&app.address, &bob, &code).await; // Bob stays Observer
    let bob_id = get_user_id(&app.address, &bob).await;

    let incident_id = create_incident(&app.address, &alice, team_id).await;

    let res = client
        .post(format!(
            "{}/teams/{team_id}/incidents/{incident_id}/assign",
            app.address
        ))
        .bearer_auth(&alice)
        .json(&json!({ "user_id": bob_id }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn observer_cannot_assign() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let bob = register_and_login(&app.address, "bob@example.com").await;
    let client = reqwest::Client::new();

    let team_id = create_team(&app.address, &alice, "Ops").await;
    let code = create_invitation(&app.address, &alice, team_id).await;
    join_team(&app.address, &bob, &code).await;
    let bob_id = get_user_id(&app.address, &bob).await;

    let incident_id = create_incident(&app.address, &alice, team_id).await;

    let res = client
        .post(format!(
            "{}/teams/{team_id}/incidents/{incident_id}/assign",
            app.address
        ))
        .bearer_auth(&bob)
        .json(&json!({ "user_id": bob_id }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn reassignment_replaces_previous() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let bob = register_and_login(&app.address, "bob@example.com").await;
    let charlie = register_and_login(&app.address, "charlie@example.com").await;
    let client = reqwest::Client::new();

    let team_id = create_team(&app.address, &alice, "Ops").await;
    let code = create_invitation(&app.address, &alice, team_id).await;
    join_team(&app.address, &bob, &code).await;
    let code2 = create_invitation(&app.address, &alice, team_id).await;
    join_team(&app.address, &charlie, &code2).await;

    let bob_id = get_user_id(&app.address, &bob).await;
    let charlie_id = get_user_id(&app.address, &charlie).await;
    promote_to_responder(&app.address, &alice, team_id, bob_id).await;
    promote_to_responder(&app.address, &alice, team_id, charlie_id).await;

    let incident_id = create_incident(&app.address, &alice, team_id).await;

    client
        .post(format!(
            "{}/teams/{team_id}/incidents/{incident_id}/assign",
            app.address
        ))
        .bearer_auth(&alice)
        .json(&json!({ "user_id": bob_id }))
        .send()
        .await
        .unwrap();

    let res = client
        .post(format!(
            "{}/teams/{team_id}/incidents/{incident_id}/assign",
            app.address
        ))
        .bearer_auth(&alice)
        .json(&json!({ "user_id": charlie_id }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn cannot_assign_non_member() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let client = reqwest::Client::new();

    let team_id = create_team(&app.address, &alice, "Ops").await;
    let incident_id = create_incident(&app.address, &alice, team_id).await;

    let random_id = Uuid::new_v4();

    let res = client
        .post(format!(
            "{}/teams/{team_id}/incidents/{incident_id}/assign",
            app.address
        ))
        .bearer_auth(&alice)
        .json(&json!({ "user_id": random_id }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}
