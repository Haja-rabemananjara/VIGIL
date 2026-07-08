use hmac::{Hmac, Mac};
use serde_json::json;
use sha2::Sha256;

mod common;
use common::spawn_app;

type HmacSha256 = Hmac<Sha256>;

fn sign_payload(secret: &str, body: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(body);
    let result = mac.finalize();
    format!("sha256={}", hex::encode(result.into_bytes()))
}

#[tokio::test]
async fn webhook_valid_signature_returns_202() {
    let app = spawn_app().await;

    let payload = json!({
        "action": "completed",
        "workflow_run": {
            "conclusion": "failure",
            "name": "CI",
            "html_url": "https://github.com/org/repo/actions/runs/123"
        },
        "repository": { "full_name": "org/repo" }
    });

    let body = serde_json::to_vec(&payload).unwrap();
    let signature = sign_payload("test-webhook-secret", &body);

    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/webhooks/github", app.address))
        .header("content-type", "application/json")
        .header("x-hub-signature-256", &signature)
        .header("x-github-event", "workflow_run")
        .body(body)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 202);
}

#[tokio::test]
async fn webhook_invalid_signature_returns_401() {
    let app = spawn_app().await;

    let payload = json!({ "action": "completed" });
    let body = serde_json::to_vec(&payload).unwrap();

    let bad_signature = sign_payload("wrong-secret", &body);

    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/webhooks/github", app.address))
        .header("content-type", "application/json")
        .header("x-hub-signature-256", &bad_signature)
        .header("x-github-event", "workflow_run")
        .body(body)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn webhook_missing_signature_returns_401() {
    let app = spawn_app().await;

    let payload = json!({ "action": "completed" });
    let body = serde_json::to_vec(&payload).unwrap();

    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/webhooks/github", app.address))
        .header("content-type", "application/json")
        .header("x-github-event", "workflow_run")
        .body(body)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn webhook_valid_delivery_is_persisted() {
    let app = spawn_app().await;

    let payload = json!({
        "action": "completed",
        "workflow_run": { "conclusion": "failure" }
    });

    let body = serde_json::to_vec(&payload).unwrap();
    let signature = sign_payload("test-webhook-secret", &body);

    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/webhooks/github", app.address))
        .header("content-type", "application/json")
        .header("x-hub-signature-256", &signature)
        .header("x-github-event", "workflow_run")
        .body(body)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 202);

    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    let row =
        sqlx::query!("SELECT service, event_type, hmac_valid FROM webhook_deliveries LIMIT 1")
            .fetch_one(&app.pool)
            .await
            .unwrap();

    assert_eq!(row.service, "github");
    assert_eq!(row.event_type, "workflow_run");
    assert_eq!(row.hmac_valid, Some(true));
}

#[tokio::test]
async fn webhook_invalid_signature_does_not_persist() {
    let app = spawn_app().await;

    let payload = json!({ "action": "completed" });
    let body = serde_json::to_vec(&payload).unwrap();
    let bad_signature = sign_payload("wrong-secret", &body);

    let client = reqwest::Client::new();
    client
        .post(format!("{}/webhooks/github", app.address))
        .header("content-type", "application/json")
        .header("x-hub-signature-256", &bad_signature)
        .header("x-github-event", "workflow_run")
        .body(body)
        .send()
        .await
        .unwrap();

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM webhook_deliveries")
        .fetch_one(&app.pool)
        .await
        .unwrap();

    assert_eq!(count.0, 0);
}
