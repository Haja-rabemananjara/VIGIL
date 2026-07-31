use hmac::{Hmac, Mac};
use serde_json::json;
use sha2::Sha256;
use uuid::Uuid;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

mod common;
use common::spawn_app;

type HmacSha256 = Hmac<Sha256>;

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

async fn connect_discord(address: &str, token: &str, webhook_url: &str) {
    let client = reqwest::Client::new();
    let res = client
        .post(format!("{address}/me/services/discord"))
        .bearer_auth(token)
        .json(&json!({ "token": webhook_url }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200, "discord connection failed");
}

async fn create_rule(
    address: &str,
    token: &str,
    team_id: Uuid,
    rule_json: serde_json::Value,
) -> Uuid {
    let client = reqwest::Client::new();
    let res = client
        .post(format!("{address}/teams/{team_id}/rules"))
        .bearer_auth(token)
        .json(&rule_json)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201, "rule creation failed");
    let body: serde_json::Value = res.json().await.unwrap();
    Uuid::parse_str(body["id"].as_str().unwrap()).unwrap()
}

fn sign_payload(secret: &str, body: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(body);
    let result = mac.finalize();
    format!("sha256={}", hex::encode(result.into_bytes()))
}

async fn send_github_webhook(address: &str, event: &str, payload: serde_json::Value) {
    let body = serde_json::to_vec(&payload).unwrap();
    let signature = sign_payload("test-webhook-secret", &body);
    let res = reqwest::Client::new()
        .post(format!("{address}/webhooks/github"))
        .header("content-type", "application/json")
        .header("x-hub-signature-256", &signature)
        .header("x-github-event", event)
        .body(body)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 202);
}

async fn list_incidents(address: &str, token: &str, team_id: Uuid) -> Vec<serde_json::Value> {
    let res = reqwest::Client::new()
        .get(format!("{address}/teams/{team_id}/incidents"))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = res.json().await.unwrap();
    body["incidents"].as_array().unwrap().clone()
}

#[tokio::test]
async fn phase2_core_full_demo_github_to_incident_and_discord() {
    let app = spawn_app().await;

    let mock_discord = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/discord-webhook"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_discord)
        .await;
    let discord_webhook_url = format!("{}/discord-webhook", mock_discord.uri());

    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops").await;
    connect_discord(&app.address, &alice, &discord_webhook_url).await;

    create_rule(
        &app.address,
        &alice,
        team_id,
        json!({
            "name": "CI failure > critical incident",
            "enabled": true,
            "trigger": {
                "service": "github",
                "event": "workflow_run",
                "filters": {
                    "workflow_run.conclusion": "failure"
                }
            },
            "reaction": {
                "type": "vigil_create_incident",
                "payload": {
                    "title": "CI broken on {{repository.name}}",
                    "severity": "high",
                    "body": "Workflow {{workflow_run.name}} failed - investigate ASAP"
                }
            }
        }),
    )
    .await;

    create_rule(
        &app.address,
        &alice,
        team_id,
        json!({
            "name": "CI failure > notify Discord",
            "enabled": true,
            "trigger": {
                "service": "github",
                "event": "workflow_run",
                "filters": {
                    "workflow_run.conclusion": "failure"
                }
            },
            "reaction": {
                "type": "discord_message",
                "payload": {
                    "content": "CI broken on {{repository.name}} - workflow {{workflow_run.name}}",
                    "username": "VIGIL"
                }
            }
        }),
    )
    .await;

    send_github_webhook(
        &app.address,
        "workflow_run",
        json!({
            "action": "completed",
            "workflow_run": {
                "name": "CI",
                "conclusion": "failure",
                "html_url": "https://github.com/hajatiana/vigil/actions/runs/12345"
            },
            "repository": {
                "name": "vigil",
                "full_name": "hajatiana/vigil"
            }
        }),
    )
    .await;

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    let mut both_ran = false;
    while std::time::Instant::now() < deadline {
        let incidents = list_incidents(&app.address, &alice, team_id).await;
        let discord_calls = mock_discord.received_requests().await.unwrap_or_default();
        if !incidents.is_empty() && !discord_calls.is_empty() {
            both_ran = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    assert!(
        both_ran,
        "both the incident AND the Discord message should be present within 5s"
    );

    let incidents = list_incidents(&app.address, &alice, team_id).await;
    assert_eq!(incidents.len(), 1, "exactly one incident expected");
    let incident = &incidents[0];
    assert_eq!(incident["title"], "CI broken on vigil");
    assert_eq!(incident["severity"], "high");
    assert_eq!(incident["body"], "Workflow CI failed - investigate ASAP");
    assert_eq!(incident["status"], "open");

    let discord_calls = mock_discord.received_requests().await.unwrap();
    assert_eq!(
        discord_calls.len(),
        1,
        "exactly one Discord message expected"
    );
    let discord_body: serde_json::Value = serde_json::from_slice(&discord_calls[0].body).unwrap();
    assert_eq!(discord_body["content"], "CI broken on vigil - workflow CI");
    assert_eq!(discord_body["username"], "VIGIL");
}
