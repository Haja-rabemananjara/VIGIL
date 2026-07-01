use serde_json::json;
use uuid::Uuid;
use futures_util::StreamExt;
use tokio_tungstenite::{connect_async, tungstenite::Message};

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

fn ws_url(http_address: &str, token: &str) -> String {
    let base = http_address.replace("http://", "ws://");
    format!("{base}/ws?token={token}")
}

async fn wait_for_event(
    ws: &mut (impl StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin),
    expected_type: &str,
) -> serde_json::Value {
    let timeout = tokio::time::Duration::from_secs(5);
    let deadline = tokio::time::Instant::now() + timeout;

    loop {
        let remaining = deadline - tokio::time::Instant::now();
        match tokio::time::timeout(remaining, ws.next()).await {
            Ok(Some(Ok(Message::Text(text)))) => {
                let event: serde_json::Value =
                    serde_json::from_str(&text).expect("invalid JSON from WS");
                if event["type"].as_str() == Some(expected_type) {
                    return event;
                }
            }
            Ok(Some(Ok(_))) => continue,
            _ => panic!("timed out waiting for WS event '{expected_type}'"),
        }
    }
}

#[tokio::test]
async fn state_change_broadcasts_to_team_member() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let bob = register_and_login(&app.address, "bob@example.com").await;

    let team_id = create_team(&app.address, &alice, "Ops").await;
    let code = create_invitation(&app.address, &alice, team_id).await;
    join_team(&app.address, &bob, &code).await;

    let incident_id = create_incident(&app.address, &alice, team_id).await;

    let url = ws_url(&app.address, &bob);
    let (mut bob_ws, _) = connect_async(&url).await.unwrap();

    let _hello = bob_ws.next().await.unwrap().unwrap();

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

    let event = wait_for_event(&mut bob_ws, "incident_state_changed").await;
    assert_eq!(event["incident_id"], incident_id.to_string());
    assert_eq!(event["new_state"], "acknowledged");
}

#[tokio::test]
async fn non_member_does_not_receive_team_events() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let charlie = register_and_login(&app.address, "charlie@example.com").await;

    let team_id = create_team(&app.address, &alice, "Ops").await;
    let incident_id = create_incident(&app.address, &alice, team_id).await;

    let url = ws_url(&app.address, &charlie);
    let (mut charlie_ws, _) = connect_async(&url).await.unwrap();
    let _hello = charlie_ws.next().await.unwrap().unwrap();

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

    let timeout = tokio::time::Duration::from_secs(2);
    let result = tokio::time::timeout(timeout, charlie_ws.next()).await;
    assert!(result.is_err(), "charlie should not receive team events");
}

#[tokio::test]
async fn escalation_emits_both_events() {
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

    let url = ws_url(&app.address, &alice);
    let (mut alice_ws, _) = connect_async(&url).await.unwrap();
    let _hello = alice_ws.next().await.unwrap().unwrap();

    client
        .patch(format!(
            "{}/teams/{team_id}/incidents/{incident_id}/status",
            app.address
        ))
        .bearer_auth(&alice)
        .json(&json!({ "status": "escalated", "severity": "critical" }))
        .send()
        .await
        .unwrap();

    let event1 = wait_for_event(&mut alice_ws, "incident_state_changed").await;
    assert_eq!(event1["new_state"], "escalated");

    let event2 = wait_for_event(&mut alice_ws, "incident_escalated").await;
    assert_eq!(event2["new_severity"], "critical");
}

#[tokio::test]
async fn timeline_entry_broadcasts_to_team() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let bob = register_and_login(&app.address, "bob@example.com").await;

    let team_id = create_team(&app.address, &alice, "Ops").await;
    let code = create_invitation(&app.address, &alice, team_id).await;
    join_team(&app.address, &bob, &code).await;

    let incident_id = create_incident(&app.address, &alice, team_id).await;

    let url = ws_url(&app.address, &bob);
    let (mut bob_ws, _) = connect_async(&url).await.unwrap();
    let _hello = bob_ws.next().await.unwrap().unwrap();

    let client = reqwest::Client::new();
    client
        .post(format!(
            "{}/teams/{team_id}/incidents/{incident_id}/timeline",
            app.address
        ))
        .bearer_auth(&alice)
        .json(&json!({ "content": "Investigating root cause" }))
        .send()
        .await
        .unwrap();

    let event = wait_for_event(&mut bob_ws, "timeline_entry_added").await;
    assert_eq!(event["incident_id"], incident_id.to_string());
    assert_eq!(event["content"], "Investigating root cause");
}
