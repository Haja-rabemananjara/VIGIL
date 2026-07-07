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

async fn create_release(
    address: &str,
    token: &str,
    team_id: Uuid,
    title: &str,
    steps: &[&str],
) -> (String, Vec<String>) {
    let client = reqwest::Client::new();
    let res = client
        .post(format!("{address}/teams/{team_id}/releases"))
        .bearer_auth(token)
        .json(&json!({ "title": title, "steps": steps }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let body: serde_json::Value = res.json().await.unwrap();
    let release_id = body["id"].as_str().unwrap().to_string();
    let step_ids: Vec<String> = body["steps"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["id"].as_str().unwrap().to_string())
        .collect();
    (release_id, step_ids)
}

async fn start_release(address: &str, token: &str, team_id: Uuid, release_id: &str) {
    let client = reqwest::Client::new();
    let res = client
        .post(format!(
            "{address}/teams/{team_id}/releases/{release_id}/start"
        ))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

async fn create_incident(address: &str, token: &str, team_id: Uuid, title: &str) -> String {
    let client = reqwest::Client::new();
    let res = client
        .post(format!("{address}/teams/{team_id}/incidents"))
        .bearer_auth(token)
        .json(&json!({ "title": title, "severity": "high" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let body: serde_json::Value = res.json().await.unwrap();
    body["id"].as_str().unwrap().to_string()
}

async fn resolve_incident(address: &str, token: &str, team_id: Uuid, incident_id: &str) {
    let client = reqwest::Client::new();
    client
        .patch(format!(
            "{address}/teams/{team_id}/incidents/{incident_id}/status"
        ))
        .bearer_auth(token)
        .json(&json!({ "status": "acknowledged" }))
        .send()
        .await
        .unwrap();
    client
        .patch(format!(
            "{address}/teams/{team_id}/incidents/{incident_id}/status"
        ))
        .bearer_auth(token)
        .json(&json!({ "status": "resolved" }))
        .send()
        .await
        .unwrap();
}

async fn get_release(
    address: &str,
    token: &str,
    team_id: Uuid,
    release_id: &str,
) -> serde_json::Value {
    let client = reqwest::Client::new();
    let res = client
        .get(format!("{address}/teams/{team_id}/releases/{release_id}"))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    res.json().await.unwrap()
}

#[tokio::test]
async fn link_incident_auto_blocks_in_progress_release() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let client = reqwest::Client::new();
    let team_id = create_team(&app.address, &alice, "Ops Team").await;

    let (release_id, _) =
        create_release(&app.address, &alice, team_id, "v1.0", &["build", "deploy"]).await;
    start_release(&app.address, &alice, team_id, &release_id).await;

    let incident_id = create_incident(&app.address, &alice, team_id, "DB is down").await;

    let res = client
        .post(format!(
            "{}/teams/{team_id}/releases/{release_id}/link",
            app.address
        ))
        .bearer_auth(&alice)
        .json(&json!({ "incident_id": incident_id }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["status"], "blocked");
}

#[tokio::test]
async fn link_resolved_incident_does_not_block() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let client = reqwest::Client::new();
    let team_id = create_team(&app.address, &alice, "Ops Team").await;

    let (release_id, _) = create_release(&app.address, &alice, team_id, "v1.0", &["build"]).await;
    start_release(&app.address, &alice, team_id, &release_id).await;

    let incident_id = create_incident(&app.address, &alice, team_id, "Fixed issue").await;
    resolve_incident(&app.address, &alice, team_id, &incident_id).await;

    let res = client
        .post(format!(
            "{}/teams/{team_id}/releases/{release_id}/link",
            app.address
        ))
        .bearer_auth(&alice)
        .json(&json!({ "incident_id": incident_id }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["status"], "in_progress");
}

#[tokio::test]
async fn link_incident_to_created_release_no_block() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let client = reqwest::Client::new();
    let team_id = create_team(&app.address, &alice, "Ops Team").await;

    let (release_id, _) = create_release(&app.address, &alice, team_id, "v1.0", &["build"]).await;

    let incident_id = create_incident(&app.address, &alice, team_id, "Issue").await;

    let res = client
        .post(format!(
            "{}/teams/{team_id}/releases/{release_id}/link",
            app.address
        ))
        .bearer_auth(&alice)
        .json(&json!({ "incident_id": incident_id }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["status"], "created"); // stays created, not blocked
}

#[tokio::test]
async fn duplicate_link_returns_409() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let client = reqwest::Client::new();
    let team_id = create_team(&app.address, &alice, "Ops Team").await;

    let (release_id, _) = create_release(&app.address, &alice, team_id, "v1.0", &["build"]).await;
    let incident_id = create_incident(&app.address, &alice, team_id, "Issue").await;

    client
        .post(format!(
            "{}/teams/{team_id}/releases/{release_id}/link",
            app.address
        ))
        .bearer_auth(&alice)
        .json(&json!({ "incident_id": incident_id }))
        .send()
        .await
        .unwrap();

    let res = client
        .post(format!(
            "{}/teams/{team_id}/releases/{release_id}/link",
            app.address
        ))
        .bearer_auth(&alice)
        .json(&json!({ "incident_id": incident_id }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn resolve_incident_auto_unblocks_release() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let client = reqwest::Client::new();
    let team_id = create_team(&app.address, &alice, "Ops Team").await;

    let (release_id, _) =
        create_release(&app.address, &alice, team_id, "v1.0", &["build", "deploy"]).await;
    start_release(&app.address, &alice, team_id, &release_id).await;

    let incident_id = create_incident(&app.address, &alice, team_id, "DB is down").await;

    client
        .post(format!(
            "{}/teams/{team_id}/releases/{release_id}/link",
            app.address
        ))
        .bearer_auth(&alice)
        .json(&json!({ "incident_id": incident_id }))
        .send()
        .await
        .unwrap();

    let release = get_release(&app.address, &alice, team_id, &release_id).await;
    assert_eq!(release["status"], "blocked");

    resolve_incident(&app.address, &alice, team_id, &incident_id).await;

    let release = get_release(&app.address, &alice, team_id, &release_id).await;
    assert_eq!(release["status"], "in_progress");
}

#[tokio::test]
async fn multi_incident_only_unblocks_when_all_resolved() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let client = reqwest::Client::new();
    let team_id = create_team(&app.address, &alice, "Ops Team").await;

    let (release_id, _) = create_release(&app.address, &alice, team_id, "v1.0", &["build"]).await;
    start_release(&app.address, &alice, team_id, &release_id).await;

    let incident_1 = create_incident(&app.address, &alice, team_id, "Issue 1").await;
    let incident_2 = create_incident(&app.address, &alice, team_id, "Issue 2").await;

    client
        .post(format!(
            "{}/teams/{team_id}/releases/{release_id}/link",
            app.address
        ))
        .bearer_auth(&alice)
        .json(&json!({ "incident_id": incident_1 }))
        .send()
        .await
        .unwrap();

    client
        .post(format!(
            "{}/teams/{team_id}/releases/{release_id}/link",
            app.address
        ))
        .bearer_auth(&alice)
        .json(&json!({ "incident_id": incident_2 }))
        .send()
        .await
        .unwrap();

    let release = get_release(&app.address, &alice, team_id, &release_id).await;
    assert_eq!(release["status"], "blocked");

    resolve_incident(&app.address, &alice, team_id, &incident_1).await;

    let release = get_release(&app.address, &alice, team_id, &release_id).await;
    assert_eq!(release["status"], "blocked");

    resolve_incident(&app.address, &alice, team_id, &incident_2).await;

    let release = get_release(&app.address, &alice, team_id, &release_id).await;
    assert_eq!(release["status"], "in_progress");
}

#[tokio::test]
async fn unlink_incident_auto_unblocks() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let client = reqwest::Client::new();
    let team_id = create_team(&app.address, &alice, "Ops Team").await;

    let (release_id, _) = create_release(&app.address, &alice, team_id, "v1.0", &["build"]).await;
    start_release(&app.address, &alice, team_id, &release_id).await;

    let incident_id = create_incident(&app.address, &alice, team_id, "Issue").await;

    client
        .post(format!(
            "{}/teams/{team_id}/releases/{release_id}/link",
            app.address
        ))
        .bearer_auth(&alice)
        .json(&json!({ "incident_id": incident_id }))
        .send()
        .await
        .unwrap();

    let res = client
        .post(format!(
            "{}/teams/{team_id}/releases/{release_id}/unlink",
            app.address
        ))
        .bearer_auth(&alice)
        .json(&json!({ "incident_id": incident_id }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["status"], "in_progress");
}

#[tokio::test]
async fn validate_step_on_blocked_release_returns_409() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let client = reqwest::Client::new();
    let team_id = create_team(&app.address, &alice, "Ops Team").await;

    let (release_id, step_ids) =
        create_release(&app.address, &alice, team_id, "v1.0", &["build", "deploy"]).await;
    start_release(&app.address, &alice, team_id, &release_id).await;

    let incident_id = create_incident(&app.address, &alice, team_id, "Blocker").await;

    client
        .post(format!(
            "{}/teams/{team_id}/releases/{release_id}/link",
            app.address
        ))
        .bearer_auth(&alice)
        .json(&json!({ "incident_id": incident_id }))
        .send()
        .await
        .unwrap();

    let res = client
        .post(format!(
            "{}/teams/{team_id}/releases/{release_id}/steps/{}/validate",
            app.address, step_ids[0]
        ))
        .bearer_auth(&alice)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn unlink_nonexistent_link_returns_404() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let client = reqwest::Client::new();
    let team_id = create_team(&app.address, &alice, "Ops Team").await;

    let (release_id, _) = create_release(&app.address, &alice, team_id, "v1.0", &["build"]).await;
    let incident_id = create_incident(&app.address, &alice, team_id, "Issue").await;

    let res = client
        .post(format!(
            "{}/teams/{team_id}/releases/{release_id}/unlink",
            app.address
        ))
        .bearer_auth(&alice)
        .json(&json!({ "incident_id": incident_id }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}
