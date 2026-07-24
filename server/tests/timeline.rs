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

#[tokio::test]
async fn responder_can_add_timeline_entry() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops").await;
    let incident_id = create_incident(&app.address, &alice, team_id).await;
    let client = reqwest::Client::new();

    let res = client
        .post(format!(
            "{}/teams/{team_id}/incidents/{incident_id}/timeline",
            app.address
        ))
        .bearer_auth(&alice)
        .json(&json!({ "content": "Investigating the issue" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["content"], "Investigating the issue");
    assert_eq!(body["kind"], "message");
    assert!(body["id"].as_str().is_some());
    assert!(body["created_at"].as_i64().is_some());
}

#[tokio::test]
async fn observer_cannot_add_timeline_entry() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let bob = register_and_login(&app.address, "bob@example.com").await;
    let client = reqwest::Client::new();

    let team_id = create_team(&app.address, &alice, "Ops").await;
    let code = create_invitation(&app.address, &alice, team_id).await;
    join_team(&app.address, &bob, &code).await;
    let incident_id = create_incident(&app.address, &alice, team_id).await;

    let res = client
        .post(format!(
            "{}/teams/{team_id}/incidents/{incident_id}/timeline",
            app.address
        ))
        .bearer_auth(&bob)
        .json(&json!({ "content": "I am watching" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn observer_can_read_timeline() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let bob = register_and_login(&app.address, "bob@example.com").await;
    let client = reqwest::Client::new();

    let team_id = create_team(&app.address, &alice, "Ops").await;
    let code = create_invitation(&app.address, &alice, team_id).await;
    join_team(&app.address, &bob, &code).await;
    let incident_id = create_incident(&app.address, &alice, team_id).await;

    client
        .post(format!(
            "{}/teams/{team_id}/incidents/{incident_id}/timeline",
            app.address
        ))
        .bearer_auth(&alice)
        .json(&json!({ "content": "First update" }))
        .send()
        .await
        .unwrap();

    let res = client
        .get(format!(
            "{}/teams/{team_id}/incidents/{incident_id}/timeline",
            app.address
        ))
        .bearer_auth(&bob)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(!body["entries"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn content_too_long_returns_422() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops").await;
    let incident_id = create_incident(&app.address, &alice, team_id).await;
    let client = reqwest::Client::new();

    let long_content = "a".repeat(2001);

    let res = client
        .post(format!(
            "{}/teams/{team_id}/incidents/{incident_id}/timeline",
            app.address
        ))
        .bearer_auth(&alice)
        .json(&json!({ "content": long_content }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn empty_content_returns_422() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops").await;
    let incident_id = create_incident(&app.address, &alice, team_id).await;
    let client = reqwest::Client::new();

    let res = client
        .post(format!(
            "{}/teams/{team_id}/incidents/{incident_id}/timeline",
            app.address
        ))
        .bearer_auth(&alice)
        .json(&json!({ "content": "   " }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn timeline_contains_system_entries_from_transitions() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops").await;
    let incident_id = create_incident(&app.address, &alice, team_id).await;
    let client = reqwest::Client::new();

    client
        .patch(format!(
            "{}/teams/{team_id}/incidents/{incident_id}/status",
            app.address
        ))
        .bearer_auth(&alice)
        .json(&json!({ "status": "acknowledged" }))
        .send()
        .await
        .unwrap();

    let res = client
        .get(format!(
            "{}/teams/{team_id}/incidents/{incident_id}/timeline",
            app.address
        ))
        .bearer_auth(&alice)
        .send()
        .await
        .unwrap();

    let body: serde_json::Value = res.json().await.unwrap();
    let entries = body["entries"].as_array().unwrap();

    let has_system = entries.iter().any(|e| e["kind"] == "system");
    assert!(has_system);
}

#[tokio::test]
async fn non_member_cannot_read_timeline() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let charlie = register_and_login(&app.address, "charlie@example.com").await;
    let client = reqwest::Client::new();

    let team_id = create_team(&app.address, &alice, "Ops").await;
    let incident_id = create_incident(&app.address, &alice, team_id).await;

    let res = client
        .get(format!(
            "{}/teams/{team_id}/incidents/{incident_id}/timeline",
            app.address
        ))
        .bearer_auth(&charlie)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

async fn add_timeline_entry(
    address: &str,
    token: &str,
    team_id: Uuid,
    incident_id: Uuid,
    content: &str,
) -> Uuid {
    let client = reqwest::Client::new();
    let res = client
        .post(format!(
            "{address}/teams/{team_id}/incidents/{incident_id}/timeline"
        ))
        .bearer_auth(token)
        .json(&json!({ "content": content }))
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
async fn author_can_edit_own_timeline_entry() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops").await;
    let incident_id = create_incident(&app.address, &alice, team_id).await;
    let client = reqwest::Client::new();

    let entry_id = add_timeline_entry(
        &app.address,
        &alice,
        team_id,
        incident_id,
        "Original content",
    )
    .await;

    let res = client
        .patch(format!("{}/timeline/{entry_id}", app.address))
        .bearer_auth(&alice)
        .json(&json!({ "content": "Updated content" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["content"], "Updated content");
    assert!(body["edited_at"].as_i64().is_some());
    assert!(body["created_at"].as_i64().is_some());
}

#[tokio::test]
async fn cannot_edit_system_entry() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops").await;
    let incident_id = create_incident(&app.address, &alice, team_id).await;
    let client = reqwest::Client::new();

    client
        .patch(format!(
            "{}/teams/{team_id}/incidents/{incident_id}/status",
            app.address
        ))
        .bearer_auth(&alice)
        .json(&json!({ "status": "acknowledged" }))
        .send()
        .await
        .unwrap();

    let res = client
        .get(format!(
            "{}/teams/{team_id}/incidents/{incident_id}/timeline",
            app.address
        ))
        .bearer_auth(&alice)
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = res.json().await.unwrap();
    let system_entry = body["entries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["kind"] == "system")
        .expect("should have a system entry");
    let entry_id = system_entry["id"].as_str().unwrap();

    let res = client
        .patch(format!("{}/timeline/{entry_id}", app.address))
        .bearer_auth(&alice)
        .json(&json!({ "content": "Hacking the system" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn manager_cannot_edit_another_members_entry() {
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

    let entry_id = add_timeline_entry(&app.address, &bob, team_id, incident_id, "Bob's note").await;

    let res = client
        .patch(format!("{}/timeline/{entry_id}", app.address))
        .bearer_auth(&alice)
        .json(&json!({ "content": "Alice overwrites" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn edit_nonexistent_entry_returns_404() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let client = reqwest::Client::new();

    let fake_id = Uuid::new_v4();
    let res = client
        .patch(format!("{}/timeline/{fake_id}", app.address))
        .bearer_auth(&alice)
        .json(&json!({ "content": "Ghost edit" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn edit_with_empty_content_returns_422() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops").await;
    let incident_id = create_incident(&app.address, &alice, team_id).await;
    let client = reqwest::Client::new();

    let entry_id =
        add_timeline_entry(&app.address, &alice, team_id, incident_id, "Will edit this").await;

    let res = client
        .patch(format!("{}/timeline/{entry_id}", app.address))
        .bearer_auth(&alice)
        .json(&json!({ "content": "   " }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn edit_with_too_long_content_returns_422() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops").await;
    let incident_id = create_incident(&app.address, &alice, team_id).await;
    let client = reqwest::Client::new();

    let entry_id =
        add_timeline_entry(&app.address, &alice, team_id, incident_id, "Will edit this").await;

    let long_content = "x".repeat(2001);
    let res = client
        .patch(format!("{}/timeline/{entry_id}", app.address))
        .bearer_auth(&alice)
        .json(&json!({ "content": long_content }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}
