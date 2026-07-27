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

async fn create_incident(
    address: &str,
    token: &str,
    team_id: Uuid,
    title: &str,
    severity: &str,
) -> Uuid {
    let client = reqwest::Client::new();
    let res = client
        .post(format!("{address}/teams/{team_id}/incidents"))
        .bearer_auth(token)
        .json(&json!({ "title": title, "severity": severity, "body": "" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201, "incident creation failed");
    let body: serde_json::Value = res.json().await.unwrap();
    Uuid::parse_str(body["id"].as_str().unwrap()).unwrap()
}

async fn acknowledge_incident(address: &str, token: &str, team_id: Uuid, incident_id: Uuid) {
    let client = reqwest::Client::new();
    let res = client
        .patch(format!(
            "{address}/teams/{team_id}/incidents/{incident_id}/status"
        ))
        .bearer_auth(token)
        .json(&json!({ "status": "acknowledged" }))
        .send()
        .await
        .unwrap();
    assert!(
        res.status().is_success(),
        "acknowledge failed: {}",
        res.status()
    );
}

async fn create_release_in_progress(
    address: &str,
    token: &str,
    team_id: Uuid,
    name: &str,
) -> (Uuid, Vec<Uuid>) {
    let client = reqwest::Client::new();
    let res = client
        .post(format!("{address}/teams/{team_id}/releases"))
        .bearer_auth(token)
        .json(&json!({
            "title": name,
            "body": "",
            "steps": ["build", "staging"]
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201, "release creation failed");
    let body: serde_json::Value = res.json().await.unwrap();
    let release_id = Uuid::parse_str(body["id"].as_str().unwrap()).unwrap();

    let step_ids: Vec<Uuid> = body["steps"]
        .as_array()
        .expect("release response must contain a 'steps' array")
        .iter()
        .map(|s| Uuid::parse_str(s["id"].as_str().unwrap()).unwrap())
        .collect();

    let start = client
        .post(format!(
            "{address}/teams/{team_id}/releases/{release_id}/start"
        ))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    assert!(
        start.status().is_success(),
        "release start failed with status {}",
        start.status()
    );

    (release_id, step_ids)
}

async fn get_incident(
    address: &str,
    token: &str,
    team_id: Uuid,
    incident_id: Uuid,
) -> serde_json::Value {
    let client = reqwest::Client::new();
    let res = client
        .get(format!("{address}/teams/{team_id}/incidents/{incident_id}"))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    res.json().await.unwrap()
}

async fn get_release(
    address: &str,
    token: &str,
    team_id: Uuid,
    release_id: Uuid,
) -> serde_json::Value {
    let client = reqwest::Client::new();
    let res = client
        .get(format!("{address}/teams/{team_id}/releases/{release_id}"))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    res.json().await.unwrap()
}

fn sign_payload(secret: &str, body: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(body);
    let result = mac.finalize();
    format!("sha256={}", hex::encode(result.into_bytes()))
}

async fn send_webhook(address: &str, event: &str, payload: serde_json::Value) {
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

async fn wait_until<F>(mut check: F) -> bool
where
    F: FnMut() -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send>>,
{
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while std::time::Instant::now() < deadline {
        if check().await {
            return true;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    false
}

#[tokio::test]
async fn escalate_incident_reaction_transitions_status() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops").await;

    let incident_id = create_incident(&app.address, &alice, team_id, "Boot loop", "medium").await;
    acknowledge_incident(&app.address, &alice, team_id, incident_id).await;

    create_rule(
        &app.address,
        &alice,
        team_id,
        json!({
            "name": "CI failure escalates incident",
            "enabled": true,
            "trigger": {
                "service": "github",
                "event": "workflow_run",
                "filters": { "workflow_run.conclusion": "failure" }
            },
            "reaction": {
                "type": "vigil_escalate_incident",
                "payload": {
                    "incident_id": incident_id.to_string(),
                    "severity": "critical"
                }
            }
        }),
    )
    .await;

    send_webhook(
        &app.address,
        "workflow_run",
        json!({
            "workflow_run": { "conclusion": "failure", "name": "CI" }
        }),
    )
    .await;

    let escalated = wait_until(|| {
        let address = app.address.clone();
        let token = alice.clone();
        Box::pin(async move {
            let incident = get_incident(&address, &token, team_id, incident_id).await;
            incident["status"] == "escalated"
        })
    })
    .await;

    assert!(
        escalated,
        "incident should have been escalated by the reaction"
    );

    let final_state = get_incident(&app.address, &alice, team_id, incident_id).await;
    assert_eq!(final_state["severity"], "critical");
}

#[tokio::test]
async fn block_release_reaction_links_incident_and_blocks() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops").await;

    let incident_id = create_incident(&app.address, &alice, team_id, "DB slow", "high").await;
    let (release_id, _steps) =
        create_release_in_progress(&app.address, &alice, team_id, "Release 1.2").await;

    create_rule(
        &app.address,
        &alice,
        team_id,
        json!({
            "name": "CI failure blocks release",
            "enabled": true,
            "trigger": {
                "service": "github",
                "event": "workflow_run",
                "filters": { "workflow_run.conclusion": "failure" }
            },
            "reaction": {
                "type": "vigil_block_release",
                "payload": {
                    "release_id": release_id.to_string(),
                    "incident_id": incident_id.to_string()
                }
            }
        }),
    )
    .await;

    send_webhook(
        &app.address,
        "workflow_run",
        json!({ "workflow_run": { "conclusion": "failure" } }),
    )
    .await;

    let blocked = wait_until(|| {
        let address = app.address.clone();
        let token = alice.clone();
        Box::pin(async move {
            let release = get_release(&address, &token, team_id, release_id).await;
            release["status"] == "blocked"
        })
    })
    .await;

    assert!(blocked, "release should have been blocked by the reaction");
}

#[tokio::test]
async fn validate_step_reaction_progresses_release() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops").await;

    let (release_id, step_ids) =
        create_release_in_progress(&app.address, &alice, team_id, "Release 2.0").await;
    let first_step_id = step_ids[0];

    create_rule(
        &app.address,
        &alice,
        team_id,
        json!({
            "name": "CI success validates first step",
            "enabled": true,
            "trigger": {
                "service": "github",
                "event": "workflow_run",
                "filters": { "workflow_run.conclusion": "success" }
            },
            "reaction": {
                "type": "vigil_validate_release_step",
                "payload": {
                    "release_id": release_id.to_string(),
                    "step_id": first_step_id.to_string()
                }
            }
        }),
    )
    .await;

    send_webhook(
        &app.address,
        "workflow_run",
        json!({ "workflow_run": { "conclusion": "success" } }),
    )
    .await;

    let validated = wait_until(|| {
        let address = app.address.clone();
        let token = alice.clone();
        Box::pin(async move {
            let release = get_release(&address, &token, team_id, release_id).await;
            release["steps"]
                .as_array()
                .and_then(|steps| steps.iter().find(|s| s["id"] == first_step_id.to_string()))
                .map(|step| step["validated_at"].is_null())
                .unwrap_or(false)
        })
    })
    .await;

    assert!(
        validated,
        "first step should have been validated by the reaction"
    );
}
