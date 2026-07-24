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

async fn add_timeline_entry(address: &str, token: &str, team_id: Uuid, incident_id: Uuid, content: &str) -> Uuid {
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

#[tokio::test]
async fn get_available_emojis() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let client = reqwest::Client::new();

    let res = client
        .get(format!("{}/reactions/available", app.address))
        .bearer_auth(&alice)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body: serde_json::Value = res.json().await.unwrap();
    let emojis = body["emojis"].as_array().unwrap();
    assert!(emojis.len() >= 5);
    assert!(emojis.contains(&json!("+1")));
    assert!(emojis.contains(&json!("fire")));
}

#[tokio::test]
async fn add_reaction_happy_path() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops").await;
    let incident_id = create_incident(&app.address, &alice, team_id).await;
    let entry_id = add_timeline_entry(&app.address, &alice, team_id, incident_id, "Note").await;
    let client = reqwest::Client::new();

    let res = client
        .post(format!("{}/timeline/{entry_id}/reactions", app.address))
        .bearer_auth(&alice)
        .json(&json!({ "emoji": "+1" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn duplicate_reaction_returns_409() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops").await;
    let incident_id = create_incident(&app.address, &alice, team_id).await;
    let entry_id = add_timeline_entry(&app.address, &alice, team_id, incident_id, "Note").await;
    let client = reqwest::Client::new();

    client
        .post(format!("{}/timeline/{entry_id}/reactions", app.address))
        .bearer_auth(&alice)
        .json(&json!({ "emoji": "+1" }))
        .send()
        .await
        .unwrap();

    let res = client
        .post(format!("{}/timeline/{entry_id}/reactions", app.address))
        .bearer_auth(&alice)
        .json(&json!({ "emoji": "+1" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn different_emojis_on_same_entry_allowed() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops").await;
    let incident_id = create_incident(&app.address, &alice, team_id).await;
    let entry_id = add_timeline_entry(&app.address, &alice, team_id, incident_id, "Note").await;
    let client = reqwest::Client::new();

    let res1 = client
        .post(format!("{}/timeline/{entry_id}/reactions", app.address))
        .bearer_auth(&alice)
        .json(&json!({ "emoji": "+1" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res1.status(), StatusCode::CREATED);

    let res2 = client
        .post(format!("{}/timeline/{entry_id}/reactions", app.address))
        .bearer_auth(&alice)
        .json(&json!({ "emoji": "fire" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res2.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn unknown_emoji_returns_422() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops").await;
    let incident_id = create_incident(&app.address, &alice, team_id).await;
    let entry_id = add_timeline_entry(&app.address, &alice, team_id, incident_id, "Note").await;
    let client = reqwest::Client::new();

    let res = client
        .post(format!("{}/timeline/{entry_id}/reactions", app.address))
        .bearer_auth(&alice)
        .json(&json!({ "emoji": "rocket" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn remove_reaction_happy_path() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops").await;
    let incident_id = create_incident(&app.address, &alice, team_id).await;
    let entry_id = add_timeline_entry(&app.address, &alice, team_id, incident_id, "Note").await;
    let client = reqwest::Client::new();

    client
        .post(format!("{}/timeline/{entry_id}/reactions", app.address))
        .bearer_auth(&alice)
        .json(&json!({ "emoji": "check" }))
        .send()
        .await
        .unwrap();

    let res = client
        .delete(format!("{}/timeline/{entry_id}/reactions/check", app.address))
        .bearer_auth(&alice)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn remove_nonexistent_reaction_returns_404() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops").await;
    let incident_id = create_incident(&app.address, &alice, team_id).await;
    let entry_id = add_timeline_entry(&app.address, &alice, team_id, incident_id, "Note").await;
    let client = reqwest::Client::new();

    let res = client
        .delete(format!("{}/timeline/{entry_id}/reactions/+1", app.address))
        .bearer_auth(&alice)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_incident_reactions_aggregated() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops").await;
    let incident_id = create_incident(&app.address, &alice, team_id).await;
    let entry_id = add_timeline_entry(&app.address, &alice, team_id, incident_id, "Note").await;
    let client = reqwest::Client::new();

    client
        .post(format!("{}/timeline/{entry_id}/reactions", app.address))
        .bearer_auth(&alice)
        .json(&json!({ "emoji": "+1" }))
        .send()
        .await
        .unwrap();
    client
        .post(format!("{}/timeline/{entry_id}/reactions", app.address))
        .bearer_auth(&alice)
        .json(&json!({ "emoji": "fire" }))
        .send()
        .await
        .unwrap();

    let res = client
        .get(format!(
            "{}/teams/{team_id}/incidents/{incident_id}/reactions",
            app.address
        ))
        .bearer_auth(&alice)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body: serde_json::Value = res.json().await.unwrap();
    let entry_reactions = &body["reactions"][entry_id.to_string()];
    assert!(entry_reactions["+1"].as_array().unwrap().len() == 1);
    assert!(entry_reactions["fire"].as_array().unwrap().len() == 1);
}

#[tokio::test]
async fn reaction_on_nonexistent_entry_returns_404() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let client = reqwest::Client::new();

    let fake_id = Uuid::new_v4();
    let res = client
        .post(format!("{}/timeline/{fake_id}/reactions", app.address))
        .bearer_auth(&alice)
        .json(&json!({ "emoji": "+1" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}