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
async fn create_release_success() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops Team").await;
    let client = reqwest::Client::new();

    let res = client
        .post(format!("{}/teams/{team_id}/releases", app.address))
        .bearer_auth(&alice)
        .json(&json!({
            "title": "v1.0.0",
            "body": "First production release",
            "steps": ["build", "staging", "production"]
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);

    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["title"], "v1.0.0");
    assert_eq!(body["status"], "created");
    assert!(body["id"].as_str().is_some());

    let steps = body["steps"].as_array().unwrap();
    assert_eq!(steps.len(), 3);
    assert_eq!(steps[0]["name"], "build");
    assert_eq!(steps[0]["position"], 1);
    assert_eq!(steps[1]["name"], "staging");
    assert_eq!(steps[1]["position"], 2);
    assert_eq!(steps[2]["name"], "production");
    assert_eq!(steps[2]["position"], 3);

    assert!(steps[0]["validated_by"].is_null());
    assert!(steps[0]["validated_at"].is_null());

    assert_eq!(body["progress"]["completed"], 0);
    assert_eq!(body["progress"]["total"], 3);
}

#[tokio::test]
async fn observer_cannot_create_release() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let bob = register_and_login(&app.address, "bob@example.com").await;
    let client = reqwest::Client::new();

    let team_id = create_team(&app.address, &alice, "Ops Team").await;
    let code = create_invitation(&app.address, &alice, team_id).await;
    join_team(&app.address, &bob, &code).await;

    let res = client
        .post(format!("{}/teams/{team_id}/releases", app.address))
        .bearer_auth(&bob)
        .json(&json!({
            "title": "v1.0.0",
            "steps": ["build"]
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn responder_cannot_create_release() {
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
        .post(format!("{}/teams/{team_id}/releases", app.address))
        .bearer_auth(&bob)
        .json(&json!({
            "title": "v1.0.0",
            "steps": ["build"]
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn create_release_empty_title_rejected() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops Team").await;
    let client = reqwest::Client::new();

    let res = client
        .post(format!("{}/teams/{team_id}/releases", app.address))
        .bearer_auth(&alice)
        .json(&json!({
            "title": "",
            "steps": ["build"]
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn create_release_no_steps_rejected() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops Team").await;
    let client = reqwest::Client::new();

    let res = client
        .post(format!("{}/teams/{team_id}/releases", app.address))
        .bearer_auth(&alice)
        .json(&json!({
            "title": "v1.0.0",
            "steps": []
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn create_release_duplicate_step_names_rejected() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops Team").await;
    let client = reqwest::Client::new();

    let res = client
        .post(format!("{}/teams/{team_id}/releases", app.address))
        .bearer_auth(&alice)
        .json(&json!({
            "title": "v1.0.0",
            "steps": ["build", "staging", "Build"]
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn create_release_empty_step_name_rejected() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops Team").await;
    let client = reqwest::Client::new();

    let res = client
        .post(format!("{}/teams/{team_id}/releases", app.address))
        .bearer_auth(&alice)
        .json(&json!({
            "title": "v1.0.0",
            "steps": ["build", "", "production"]
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn non_member_cannot_create_release() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let charlie = register_and_login(&app.address, "charlie@example.com").await;
    let client = reqwest::Client::new();

    let team_id = create_team(&app.address, &alice, "Ops Team").await;

    let res = client
        .post(format!("{}/teams/{team_id}/releases", app.address))
        .bearer_auth(&charlie)
        .json(&json!({
            "title": "v1.0.0",
            "steps": ["build"]
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn list_releases_returns_all_for_team() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops Team").await;
    let client = reqwest::Client::new();

    // Create two releases
    for title in &["v1.0", "v2.0"] {
        client
            .post(format!("{}/teams/{team_id}/releases", app.address))
            .bearer_auth(&alice)
            .json(&json!({
                "title": title,
                "steps": ["build", "deploy"]
            }))
            .send()
            .await
            .unwrap();
    }

    let res = client
        .get(format!("{}/teams/{team_id}/releases", app.address))
        .bearer_auth(&alice)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let body: Vec<serde_json::Value> = res.json().await.unwrap();
    assert_eq!(body.len(), 2);

    assert_eq!(body[0]["title"], "v2.0");
    assert_eq!(body[1]["title"], "v1.0");

    assert_eq!(body[0]["progress"]["total"], 2);
    assert_eq!(body[0]["progress"]["completed"], 0);
}

#[tokio::test]
async fn list_releases_with_status_filter() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops Team").await;
    let client = reqwest::Client::new();

    client
        .post(format!("{}/teams/{team_id}/releases", app.address))
        .bearer_auth(&alice)
        .json(&json!({ "title": "v1.0", "steps": ["build"] }))
        .send()
        .await
        .unwrap();

    let res = client
        .get(format!(
            "{}/teams/{team_id}/releases?status=created",
            app.address
        ))
        .bearer_auth(&alice)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body: Vec<serde_json::Value> = res.json().await.unwrap();
    assert_eq!(body.len(), 1);

    let res = client
        .get(format!(
            "{}/teams/{team_id}/releases?status=in_progress",
            app.address
        ))
        .bearer_auth(&alice)
        .send()
        .await
        .unwrap();
    let body: Vec<serde_json::Value> = res.json().await.unwrap();
    assert_eq!(body.len(), 0);
}

#[tokio::test]
async fn list_releases_invalid_status_filter() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops Team").await;
    let client = reqwest::Client::new();

    let res = client
        .get(format!(
            "{}/teams/{team_id}/releases?status=banana",
            app.address
        ))
        .bearer_auth(&alice)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn list_releases_empty_team() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops Team").await;
    let client = reqwest::Client::new();

    let res = client
        .get(format!("{}/teams/{team_id}/releases", app.address))
        .bearer_auth(&alice)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body: Vec<serde_json::Value> = res.json().await.unwrap();
    assert_eq!(body.len(), 0);
}

#[tokio::test]
async fn observer_can_list_releases() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let bob = register_and_login(&app.address, "bob@example.com").await;
    let client = reqwest::Client::new();

    let team_id = create_team(&app.address, &alice, "Ops Team").await;

    // Manager creates a release
    client
        .post(format!("{}/teams/{team_id}/releases", app.address))
        .bearer_auth(&alice)
        .json(&json!({ "title": "v1.0", "steps": ["build"] }))
        .send()
        .await
        .unwrap();

    let code = create_invitation(&app.address, &alice, team_id).await;
    join_team(&app.address, &bob, &code).await;

    let res = client
        .get(format!("{}/teams/{team_id}/releases", app.address))
        .bearer_auth(&bob)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body: Vec<serde_json::Value> = res.json().await.unwrap();
    assert_eq!(body.len(), 1);
}

#[tokio::test]
async fn get_release_detail() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops Team").await;
    let client = reqwest::Client::new();

    let res = client
        .post(format!("{}/teams/{team_id}/releases", app.address))
        .bearer_auth(&alice)
        .json(&json!({
            "title": "v1.0.0",
            "body": "Production release",
            "steps": ["build", "staging", "production"]
        }))
        .send()
        .await
        .unwrap();
    let created: serde_json::Value = res.json().await.unwrap();
    let release_id = created["id"].as_str().unwrap();

    let res = client
        .get(format!(
            "{}/teams/{team_id}/releases/{release_id}",
            app.address
        ))
        .bearer_auth(&alice)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["id"], release_id);
    assert_eq!(body["title"], "v1.0.0");
    assert_eq!(body["body"], "Production release");
    assert_eq!(body["status"], "created");
    assert_eq!(body["steps"].as_array().unwrap().len(), 3);
    assert_eq!(body["progress"]["completed"], 0);
    assert_eq!(body["progress"]["total"], 3);
}

#[tokio::test]
async fn get_release_not_found() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops Team").await;
    let client = reqwest::Client::new();

    let fake_id = Uuid::new_v4();
    let res = client
        .get(format!(
            "{}/teams/{team_id}/releases/{fake_id}",
            app.address
        ))
        .bearer_auth(&alice)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn release_from_other_team_returns_404() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let bob = register_and_login(&app.address, "bob@example.com").await;
    let client = reqwest::Client::new();

    let alice_team = create_team(&app.address, &alice, "Alice's Team").await;
    let res = client
        .post(format!("{}/teams/{alice_team}/releases", app.address))
        .bearer_auth(&alice)
        .json(&json!({ "title": "v1.0", "steps": ["build"] }))
        .send()
        .await
        .unwrap();
    let created: serde_json::Value = res.json().await.unwrap();
    let release_id = created["id"].as_str().unwrap();

    let bob_team = create_team(&app.address, &bob, "Bob's Team").await;

    let res = client
        .get(format!(
            "{}/teams/{bob_team}/releases/{release_id}",
            app.address
        ))
        .bearer_auth(&bob)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn unauthenticated_request_returns_401() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops Team").await;
    let client = reqwest::Client::new();

    let res = client
        .get(format!("{}/teams/{team_id}/releases", app.address))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}
