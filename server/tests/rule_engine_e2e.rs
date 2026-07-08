use hmac::{Hmac, Mac};
use serde_json::json;
use sha2::Sha256;
use uuid::Uuid;

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

async fn list_incidents(address: &str, token: &str, team_id: Uuid) -> Vec<serde_json::Value> {
    let res = reqwest::Client::new()
        .get(format!("{address}/teams/{team_id}/incidents"))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = res.json().await.unwrap();
    body["incidents"]
        .as_array()
        .expect("expected 'incidents' array in response")
        .clone()
}

async fn wait_until<F>(mut check: F) -> bool
where
    F: FnMut() -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send>>,
{
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while std::time::Instant::now() < deadline {
        if check().await {
            return true;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    false
}

#[tokio::test]
async fn github_ci_failure_creates_incident_via_rule() {
    let app = spawn_app().await;

    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops").await;

    create_rule(
        &app.address,
        &alice,
        team_id,
        json!({
            "name": "CI failure → critical incident",
            "enabled": true,
            "trigger": {
                "service": "github",
                "event": "workflow_run",
                "filters": {
                    "action": "completed",
                    "workflow_run.conclusion": "failure"
                }
            },
            "reaction": {
                "type": "vigil_create_incident",
                "payload": {
                    "title": "CI broken on {{repository.name}}",
                    "severity": "high",
                    "body": "Workflow {{workflow_run.name}} failed"
                }
            }
        }),
    )
    .await;

    let list_before = list_incidents(&app.address, &alice, team_id).await;
    assert!(list_before.is_empty());

    let payload = json!({
        "action": "completed",
        "workflow_run": {
            "name": "CI",
            "conclusion": "failure",
            "html_url": "https://github.com/org/my-repo/actions/runs/12345"
        },
        "repository": {
            "name": "my-repo",
            "full_name": "org/my-repo"
        }
    });
    let body = serde_json::to_vec(&payload).unwrap();
    let signature = sign_payload("test-webhook-secret", &body);

    let res = reqwest::Client::new()
        .post(format!("{}/webhooks/github", app.address))
        .header("content-type", "application/json")
        .header("x-hub-signature-256", &signature)
        .header("x-github-event", "workflow_run")
        .body(body)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 202);

    let found = wait_until(|| {
        let address = app.address.clone();
        let token = alice.clone();
        Box::pin(async move {
            let list = list_incidents(&address, &token, team_id).await;
            !list.is_empty()
        })
    })
    .await;
    assert!(found, "engine should have created an incident within 5s");

    let list = list_incidents(&app.address, &alice, team_id).await;
    assert_eq!(list.len(), 1);
    let incident = &list[0];
    assert_eq!(incident["title"], "CI broken on my-repo");
    assert_eq!(incident["severity"], "high");
    assert_eq!(incident["body"], "Workflow CI failed");
    assert_eq!(incident["status"], "open");
}

#[tokio::test]
async fn ci_success_does_not_create_incident() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops").await;

    create_rule(
        &app.address,
        &alice,
        team_id,
        json!({
            "name": "CI failure only",
            "enabled": true,
            "trigger": {
                "service": "github",
                "event": "workflow_run",
                "filters": { "workflow_run.conclusion": "failure" }
            },
            "reaction": {
                "type": "vigil_create_incident",
                "payload": {
                    "title": "Should not fire",
                    "severity": "low"
                }
            }
        }),
    )
    .await;

    let payload = json!({
        "action": "completed",
        "workflow_run": {
            "name": "CI",
            "conclusion": "success"
        },
        "repository": { "name": "my-repo" }
    });
    let body = serde_json::to_vec(&payload).unwrap();
    let signature = sign_payload("test-webhook-secret", &body);

    reqwest::Client::new()
        .post(format!("{}/webhooks/github", app.address))
        .header("content-type", "application/json")
        .header("x-hub-signature-256", &signature)
        .header("x-github-event", "workflow_run")
        .body(body)
        .send()
        .await
        .unwrap();

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    let list = list_incidents(&app.address, &alice, team_id).await;
    assert!(list.is_empty(), "no incident should have been created");
}

#[tokio::test]
async fn disabled_rule_is_not_evaluated() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops").await;

    create_rule(
        &app.address,
        &alice,
        team_id,
        json!({
            "name": "Disabled rule",
            "enabled": false,
            "trigger": {
                "service": "github",
                "event": "workflow_run",
                "filters": { "workflow_run.conclusion": "failure" }
            },
            "reaction": {
                "type": "vigil_create_incident",
                "payload": { "title": "Nope", "severity": "low" }
            }
        }),
    )
    .await;

    let payload = json!({
        "action": "completed",
        "workflow_run": { "name": "CI", "conclusion": "failure" },
        "repository": { "name": "my-repo" }
    });
    let body = serde_json::to_vec(&payload).unwrap();
    let signature = sign_payload("test-webhook-secret", &body);

    reqwest::Client::new()
        .post(format!("{}/webhooks/github", app.address))
        .header("content-type", "application/json")
        .header("x-hub-signature-256", &signature)
        .header("x-github-event", "workflow_run")
        .body(body)
        .send()
        .await
        .unwrap();

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    let list = list_incidents(&app.address, &alice, team_id).await;
    assert!(list.is_empty(), "disabled rule must not fire");
}
