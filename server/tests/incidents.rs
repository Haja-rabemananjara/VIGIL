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
async fn manager_can_create_incident() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops Team").await;
    let client = reqwest::Client::new();

    let res = client
        .post(format!("{}/teams/{team_id}/incidents", app.address))
        .bearer_auth(&alice)
        .json(&json!({
            "title": "DB is down",
            "body": "Primary postgres instance is unreachable",
            "severity": "critical"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["title"], "DB is down");
    assert_eq!(body["severity"], "critical");
    assert_eq!(body["status"], "open");
    assert!(body["id"].as_str().is_some());
}

#[tokio::test]
async fn observer_cannot_create_incident() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let bob = register_and_login(&app.address, "bob@example.com").await;
    let client = reqwest::Client::new();

    let team_id = create_team(&app.address, &alice, "Ops Team").await;
    let code = create_invitation(&app.address, &alice, team_id).await;
    join_team(&app.address, &bob, &code).await;

    let res = client
        .post(format!("{}/teams/{team_id}/incidents", app.address))
        .bearer_auth(&bob)
        .json(&json!({ "title": "Something broke", "severity": "low" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn responder_cannot_create_incident() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let bob = register_and_login(&app.address, "bob@example.com").await;
    let client = reqwest::Client::new();

    let team_id = create_team(&app.address, &alice, "Ops Team").await;
    let code = create_invitation(&app.address, &alice, team_id).await;
    join_team(&app.address, &bob, &code).await;

    let bob_id = get_user_id(&app.address, &bob).await;
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

    let res = client
        .post(format!("{}/teams/{team_id}/incidents", app.address))
        .bearer_auth(&bob)
        .json(&json!({ "title": "Something broke", "severity": "low" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn create_incident_invalid_severity_returns_422() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops Team").await;
    let client = reqwest::Client::new();

    let res = client
        .post(format!("{}/teams/{team_id}/incidents", app.address))
        .bearer_auth(&alice)
        .json(&json!({ "title": "Something broke", "severity": "catastrophic" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn list_incidents_returns_all_for_team() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops Team").await;
    let client = reqwest::Client::new();

    for i in 0..2 {
        client
            .post(format!("{}/teams/{team_id}/incidents", app.address))
            .bearer_auth(&alice)
            .json(&json!({ "title": format!("Incident {i}"), "severity": "low" }))
            .send()
            .await
            .unwrap();
    }

    let res = client
        .get(format!("{}/teams/{team_id}/incidents", app.address))
        .bearer_auth(&alice)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["incidents"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn observer_can_list_incidents() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let bob = register_and_login(&app.address, "bob@example.com").await;
    let client = reqwest::Client::new();

    let team_id = create_team(&app.address, &alice, "Ops Team").await;
    let code = create_invitation(&app.address, &alice, team_id).await;
    join_team(&app.address, &bob, &code).await;

    client
        .post(format!("{}/teams/{team_id}/incidents", app.address))
        .bearer_auth(&alice)
        .json(&json!({ "title": "Something broke", "severity": "high" }))
        .send()
        .await
        .unwrap();

    let res = client
        .get(format!("{}/teams/{team_id}/incidents", app.address))
        .bearer_auth(&bob)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn list_incidents_filter_by_status() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops Team").await;
    let client = reqwest::Client::new();

    for _ in 0..2 {
        client
            .post(format!("{}/teams/{team_id}/incidents", app.address))
            .bearer_auth(&alice)
            .json(&json!({ "title": "An incident", "severity": "low" }))
            .send()
            .await
            .unwrap();
    }

    let res = client
        .get(format!(
            "{}/teams/{team_id}/incidents?status=open",
            app.address
        ))
        .bearer_auth(&alice)
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["incidents"].as_array().unwrap().len(), 2);

    let res = client
        .get(format!(
            "{}/teams/{team_id}/incidents?status=resolved",
            app.address
        ))
        .bearer_auth(&alice)
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["incidents"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn get_incident_detail() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops Team").await;
    let client = reqwest::Client::new();

    let res = client
        .post(format!("{}/teams/{team_id}/incidents", app.address))
        .bearer_auth(&alice)
        .json(&json!({
            "title": "Prod outage",
            "body": "All requests timing out",
            "severity": "critical"
        }))
        .send()
        .await
        .unwrap();
    let created: serde_json::Value = res.json().await.unwrap();
    let incident_id = created["id"].as_str().unwrap();

    let res = client
        .get(format!(
            "{}/teams/{team_id}/incidents/{incident_id}",
            app.address
        ))
        .bearer_auth(&alice)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["title"], "Prod outage");
    assert_eq!(body["body"], "All requests timing out");
}

#[tokio::test]
async fn non_member_cannot_see_incidents() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let charlie = register_and_login(&app.address, "charlie@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops Team").await;
    let client = reqwest::Client::new();

    let res = client
        .get(format!("{}/teams/{team_id}/incidents", app.address))
        .bearer_auth(&charlie)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn incident_from_other_team_returns_404() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let bob = register_and_login(&app.address, "bob@example.com").await;
    let client = reqwest::Client::new();

    let alice_team_id = create_team(&app.address, &alice, "Alice's Team").await;
    let res = client
        .post(format!("{}/teams/{alice_team_id}/incidents", app.address))
        .bearer_auth(&alice)
        .json(&json!({ "title": "Alice's incident", "severity": "low" }))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = res.json().await.unwrap();
    let incident_id = body["id"].as_str().unwrap();

    let bob_team_id = create_team(&app.address, &bob, "Bob's Team").await;

    let res = client
        .get(format!(
            "{}/teams/{bob_team_id}/incidents/{incident_id}",
            app.address
        ))
        .bearer_auth(&bob)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}
