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

#[tokio::test]
async fn github_ci_failure_sends_discord_message() {
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
            "name": "CI failure notifies Discord",
            "enabled": true,
            "trigger": {
                "service": "github",
                "event": "workflow_run",
                "filters": { "workflow_run.conclusion": "failure" }
            },
            "reaction": {
                "type": "discord_message",
                "payload": {
                    "content": "CI broken on {{repository.name}}",
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
            "workflow_run": {
                "conclusion": "failure",
                "name": "CI"
            },
            "repository": { "name": "my-repo" }
        }),
    )
    .await;

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while std::time::Instant::now() < deadline {
        let received = mock_discord.received_requests().await.unwrap();
        if !received.is_empty() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }

    let received = mock_discord.received_requests().await.unwrap();
    assert_eq!(
        received.len(),
        1,
        "Discord should have received exactly one message"
    );

    let request_body: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();

    assert_eq!(request_body["content"], "CI broken on my-repo");
    assert_eq!(request_body["username"], "VIGIL");
}

#[tokio::test]
async fn rule_without_discord_connection_fails_gracefully() {
    let app = spawn_app().await;

    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops").await;

    create_rule(
        &app.address,
        &alice,
        team_id,
        json!({
            "name": "Discord (unconnected) notification",
            "enabled": true,
            "trigger": {
                "service": "github",
                "event": "workflow_run",
                "filters": {}
            },
            "reaction": {
                "type": "discord_message",
                "payload": { "content": "hello" }
            }
        }),
    )
    .await;

    send_github_webhook(
        &app.address,
        "workflow_run",
        json!({ "workflow_run": { "conclusion": "success" } }),
    )
    .await;

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    let health = reqwest::Client::new()
        .get(format!("{}/health", app.address))
        .send()
        .await
        .unwrap();
    assert_eq!(health.status(), 200, "server survived reaction failure");
}

#[tokio::test]
async fn discord_5xx_response_surfaces_as_rule_failure() {
    let app = spawn_app().await;

    let mock_discord = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/discord-webhook"))
        .respond_with(ResponseTemplate::new(500))
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
            "name": "CI notifies Discord (which will fail)",
            "enabled": true,
            "trigger": {
                "service": "github",
                "event": "workflow_run",
                "filters": {}
            },
            "reaction": {
                "type": "discord_message",
                "payload": { "content": "test" }
            }
        }),
    )
    .await;

    send_github_webhook(
        &app.address,
        "workflow_run",
        json!({ "workflow_run": { "conclusion": "failure" } }),
    )
    .await;

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    let received = mock_discord.received_requests().await.unwrap();
    assert_eq!(
        received.len(),
        1,
        "Discord was called despite the eventual 500"
    );

    let health = reqwest::Client::new()
        .get(format!("{}/health", app.address))
        .send()
        .await
        .unwrap();
    assert_eq!(health.status(), 200);
}
