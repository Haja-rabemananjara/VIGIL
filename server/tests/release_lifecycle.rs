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

async fn start_release(
    address: &str,
    token: &str,
    team_id: Uuid,
    release_id: &str,
) -> reqwest::Response {
    let client = reqwest::Client::new();
    client
        .post(format!(
            "{address}/teams/{team_id}/releases/{release_id}/start"
        ))
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}

#[tokio::test]
async fn start_release_success() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops Team").await;

    let (release_id, _) =
        create_release(&app.address, &alice, team_id, "v1.0", &["build", "deploy"]).await;

    let res = start_release(&app.address, &alice, team_id, &release_id).await;
    assert_eq!(res.status(), StatusCode::OK);

    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["status"], "in_progress");
    assert!(body["started_at"].as_i64().is_some());
}

#[tokio::test]
async fn start_release_twice_fails() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops Team").await;

    let (release_id, _) = create_release(&app.address, &alice, team_id, "v1.0", &["build"]).await;

    let res = start_release(&app.address, &alice, team_id, &release_id).await;
    assert_eq!(res.status(), StatusCode::OK);

    let res = start_release(&app.address, &alice, team_id, &release_id).await;
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn observer_cannot_start_release() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let bob = register_and_login(&app.address, "bob@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops Team").await;
    let code = create_invitation(&app.address, &alice, team_id).await;
    join_team(&app.address, &bob, &code).await;

    let (release_id, _) = create_release(&app.address, &alice, team_id, "v1.0", &["build"]).await;

    let res = start_release(&app.address, &bob, team_id, &release_id).await;
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn validate_steps_sequentially() {
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

    let (release_id, step_ids) = create_release(
        &app.address,
        &alice,
        team_id,
        "v1.0",
        &["build", "staging", "prod"],
    )
    .await;

    start_release(&app.address, &alice, team_id, &release_id).await;

    let res = client
        .post(format!(
            "{}/teams/{team_id}/releases/{release_id}/steps/{}/validate",
            app.address, step_ids[0]
        ))
        .bearer_auth(&bob)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["status"], "in_progress"); // not complete yet
    assert_eq!(body["progress"]["completed"], 1);
    assert_eq!(body["progress"]["total"], 3);

    let res = client
        .post(format!(
            "{}/teams/{team_id}/releases/{release_id}/steps/{}/validate",
            app.address, step_ids[1]
        ))
        .bearer_auth(&bob)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let res = client
        .post(format!(
            "{}/teams/{team_id}/releases/{release_id}/steps/{}/validate",
            app.address, step_ids[2]
        ))
        .bearer_auth(&bob)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["status"], "completed");
    assert!(body["completed_at"].as_i64().is_some());
    assert_eq!(body["progress"]["completed"], 3);
}

#[tokio::test]
async fn validate_out_of_order_rejected() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let client = reqwest::Client::new();
    let team_id = create_team(&app.address, &alice, "Ops Team").await;

    let (release_id, step_ids) = create_release(
        &app.address,
        &alice,
        team_id,
        "v1.0",
        &["build", "staging", "prod"],
    )
    .await;

    start_release(&app.address, &alice, team_id, &release_id).await;

    let res = client
        .post(format!(
            "{}/teams/{team_id}/releases/{release_id}/steps/{}/validate",
            app.address, step_ids[1]
        ))
        .bearer_auth(&alice)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn validate_step_already_validated_returns_409() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let client = reqwest::Client::new();
    let team_id = create_team(&app.address, &alice, "Ops Team").await;

    let (release_id, step_ids) =
        create_release(&app.address, &alice, team_id, "v1.0", &["build", "deploy"]).await;

    start_release(&app.address, &alice, team_id, &release_id).await;

    client
        .post(format!(
            "{}/teams/{team_id}/releases/{release_id}/steps/{}/validate",
            app.address, step_ids[0]
        ))
        .bearer_auth(&alice)
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
async fn validate_step_on_created_release_rejected() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let client = reqwest::Client::new();
    let team_id = create_team(&app.address, &alice, "Ops Team").await;

    let (release_id, step_ids) =
        create_release(&app.address, &alice, team_id, "v1.0", &["build"]).await;

    let res = client
        .post(format!(
            "{}/teams/{team_id}/releases/{release_id}/steps/{}/validate",
            app.address, step_ids[0]
        ))
        .bearer_auth(&alice)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn observer_cannot_validate_step() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let bob = register_and_login(&app.address, "bob@example.com").await;
    let client = reqwest::Client::new();
    let team_id = create_team(&app.address, &alice, "Ops Team").await;
    let code = create_invitation(&app.address, &alice, team_id).await;
    join_team(&app.address, &bob, &code).await;

    let (release_id, step_ids) =
        create_release(&app.address, &alice, team_id, "v1.0", &["build"]).await;

    start_release(&app.address, &alice, team_id, &release_id).await;

    let res = client
        .post(format!(
            "{}/teams/{team_id}/releases/{release_id}/steps/{}/validate",
            app.address, step_ids[0]
        ))
        .bearer_auth(&bob)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn cancel_created_release() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let client = reqwest::Client::new();
    let team_id = create_team(&app.address, &alice, "Ops Team").await;

    let (release_id, _) = create_release(&app.address, &alice, team_id, "v1.0", &["build"]).await;

    let res = client
        .post(format!(
            "{}/teams/{team_id}/releases/{release_id}/cancel",
            app.address
        ))
        .bearer_auth(&alice)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["status"], "cancelled");
    assert!(body["cancelled_at"].as_i64().is_some());
}

#[tokio::test]
async fn cancel_in_progress_release() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let client = reqwest::Client::new();
    let team_id = create_team(&app.address, &alice, "Ops Team").await;

    let (release_id, _) =
        create_release(&app.address, &alice, team_id, "v1.0", &["build", "deploy"]).await;

    start_release(&app.address, &alice, team_id, &release_id).await;

    let res = client
        .post(format!(
            "{}/teams/{team_id}/releases/{release_id}/cancel",
            app.address
        ))
        .bearer_auth(&alice)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["status"], "cancelled");
}

#[tokio::test]
async fn cancel_completed_release_rejected() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let client = reqwest::Client::new();
    let team_id = create_team(&app.address, &alice, "Ops Team").await;

    let (release_id, step_ids) =
        create_release(&app.address, &alice, team_id, "v1.0", &["build"]).await;

    start_release(&app.address, &alice, team_id, &release_id).await;

    client
        .post(format!(
            "{}/teams/{team_id}/releases/{release_id}/steps/{}/validate",
            app.address, step_ids[0]
        ))
        .bearer_auth(&alice)
        .send()
        .await
        .unwrap();

    let res = client
        .post(format!(
            "{}/teams/{team_id}/releases/{release_id}/cancel",
            app.address
        ))
        .bearer_auth(&alice)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn cancel_already_cancelled_rejected() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let client = reqwest::Client::new();
    let team_id = create_team(&app.address, &alice, "Ops Team").await;

    let (release_id, _) = create_release(&app.address, &alice, team_id, "v1.0", &["build"]).await;

    client
        .post(format!(
            "{}/teams/{team_id}/releases/{release_id}/cancel",
            app.address
        ))
        .bearer_auth(&alice)
        .send()
        .await
        .unwrap();

    let res = client
        .post(format!(
            "{}/teams/{team_id}/releases/{release_id}/cancel",
            app.address
        ))
        .bearer_auth(&alice)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn validate_step_on_cancelled_release_rejected() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let client = reqwest::Client::new();
    let team_id = create_team(&app.address, &alice, "Ops Team").await;

    let (release_id, step_ids) =
        create_release(&app.address, &alice, team_id, "v1.0", &["build"]).await;

    start_release(&app.address, &alice, team_id, &release_id).await;

    client
        .post(format!(
            "{}/teams/{team_id}/releases/{release_id}/cancel",
            app.address
        ))
        .bearer_auth(&alice)
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
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn start_cancelled_release_rejected() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let client = reqwest::Client::new();
    let team_id = create_team(&app.address, &alice, "Ops Team").await;

    let (release_id, _) = create_release(&app.address, &alice, team_id, "v1.0", &["build"]).await;

    // Cancel it
    client
        .post(format!(
            "{}/teams/{team_id}/releases/{release_id}/cancel",
            app.address
        ))
        .bearer_auth(&alice)
        .send()
        .await
        .unwrap();

    let res = start_release(&app.address, &alice, team_id, &release_id).await;
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}
