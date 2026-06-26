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

async fn patch_status(
    address: &str,
    token: &str,
    team_id: Uuid,
    incident_id: Uuid,
    body: serde_json::Value,
) -> reqwest::Response {
    let client = reqwest::Client::new();
    client
        .patch(format!(
            "{address}/teams/{team_id}/incidents/{incident_id}/status"
        ))
        .bearer_auth(token)
        .json(&body)
        .send()
        .await
        .unwrap()
}

#[tokio::test]
async fn open_to_acknowledged() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops").await;
    let incident_id = create_incident(&app.address, &alice, team_id).await;

    let res = patch_status(
        &app.address,
        &alice,
        team_id,
        incident_id,
        json!({ "status": "acknowledged" }),
    )
    .await;

    assert_eq!(res.status(), StatusCode::OK);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["status"], "acknowledged");
    assert!(body["acknowledged_at"].as_i64().is_some());
}

#[tokio::test]
async fn acknowledged_to_escalated_with_severity() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops").await;
    let incident_id = create_incident(&app.address, &alice, team_id).await;

    patch_status(
        &app.address,
        &alice,
        team_id,
        incident_id,
        json!({ "status": "acknowledged" }),
    )
    .await;

    let res = patch_status(
        &app.address,
        &alice,
        team_id,
        incident_id,
        json!({ "status": "escalated", "severity": "critical" }),
    )
    .await;

    assert_eq!(res.status(), StatusCode::OK);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["status"], "escalated");
    assert_eq!(body["severity"], "critical");
    assert!(body["escalated_at"].as_i64().is_some());
}

#[tokio::test]
async fn acknowledged_to_resolved_shortcut() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops").await;
    let incident_id = create_incident(&app.address, &alice, team_id).await;

    patch_status(
        &app.address,
        &alice,
        team_id,
        incident_id,
        json!({ "status": "acknowledged" }),
    )
    .await;

    let res = patch_status(
        &app.address,
        &alice,
        team_id,
        incident_id,
        json!({ "status": "resolved" }),
    )
    .await;

    assert_eq!(res.status(), StatusCode::OK);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["status"], "resolved");
    assert!(body["resolved_at"].as_i64().is_some());
}

#[tokio::test]
async fn invalid_transition_returns_422() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops").await;
    let incident_id = create_incident(&app.address, &alice, team_id).await;

    let res = patch_status(
        &app.address,
        &alice,
        team_id,
        incident_id,
        json!({ "status": "resolved" }),
    )
    .await;

    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn open_to_escalated_skipping_acknowledged_returns_422() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops").await;
    let incident_id = create_incident(&app.address, &alice, team_id).await;

    let res = patch_status(
        &app.address,
        &alice,
        team_id,
        incident_id,
        json!({ "status": "escalated" }),
    )
    .await;

    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn observer_cannot_transition() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let bob = register_and_login(&app.address, "bob@example.com").await;

    let team_id = create_team(&app.address, &alice, "Ops").await;
    let code = create_invitation(&app.address, &alice, team_id).await;
    join_team(&app.address, &bob, &code).await; // Bob = Observer

    let incident_id = create_incident(&app.address, &alice, team_id).await;

    let res = patch_status(
        &app.address,
        &bob,
        team_id,
        incident_id,
        json!({ "status": "acknowledged" }),
    )
    .await;

    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn responder_can_acknowledge() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let bob = register_and_login(&app.address, "bob@example.com").await;

    let team_id = create_team(&app.address, &alice, "Ops").await;
    let code = create_invitation(&app.address, &alice, team_id).await;
    join_team(&app.address, &bob, &code).await;

    let bob_id = get_user_id(&app.address, &bob).await;
    let client = reqwest::Client::new();
    client
        .patch(format!(
            "{}/teams/{team_id}/members/{bob_id}/role",
            app.address
        ))
        .bearer_auth(&alice)
        .json(&json!({ "role": "responder" }))
        .send()
        .await
        .unwrap();

    let incident_id = create_incident(&app.address, &alice, team_id).await;

    let res = patch_status(
        &app.address,
        &bob,
        team_id,
        incident_id,
        json!({ "status": "acknowledged" }),
    )
    .await;

    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn severity_can_be_updated_independently() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops").await;
    let incident_id = create_incident(&app.address, &alice, team_id).await;
    let client = reqwest::Client::new();

    let res = client
        .patch(format!(
            "{}/teams/{team_id}/incidents/{incident_id}/severity",
            app.address
        ))
        .bearer_auth(&alice)
        .json(&json!({ "severity": "high" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["severity"], "high");
    assert_eq!(body["status"], "open");
}

#[tokio::test]
async fn unknown_status_returns_422() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops").await;
    let incident_id = create_incident(&app.address, &alice, team_id).await;

    let res = patch_status(
        &app.address,
        &alice,
        team_id,
        incident_id,
        json!({ "status": "on_fire" }),
    )
    .await;

    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}
