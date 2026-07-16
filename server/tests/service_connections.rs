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
async fn connect_service_returns_connection_metadata() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;

    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/me/services/github", app.address))
        .bearer_auth(&alice)
        .json(&json!({ "token": "ghp_MySecretGitHubToken123456" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);

    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["service"], "github");
    assert!(Uuid::parse_str(body["id"].as_str().unwrap()).is_ok());
    assert!(body["created_at"].as_i64().is_some());
    assert!(body["updated_at"].as_i64().is_some());
}

#[tokio::test]
async fn connect_response_does_not_contain_token() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let secret_token = "ghp_UNIQUE_SEARCHABLE_TOKEN_9876543210";

    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/me/services/github", app.address))
        .bearer_auth(&alice)
        .json(&json!({ "token": secret_token }))
        .send()
        .await
        .unwrap();

    let body_text = res.text().await.unwrap();
    assert!(
        !body_text.contains(secret_token),
        "Response body must never contain the plaintext token, got: {body_text}"
    );
    assert!(
        !body_text.contains("token"),
        "Response body should not have a 'token' field, got: {body_text}"
    );
}

#[tokio::test]
async fn stored_token_is_encrypted_not_plaintext() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let alice_id = get_user_id(&app.address, &alice).await;
    let secret_token = "ghp_ANOTHER_UNIQUE_TOKEN_ABCDEF12345";

    let client = reqwest::Client::new();
    client
        .post(format!("{}/me/services/github", app.address))
        .bearer_auth(&alice)
        .json(&json!({ "token": secret_token }))
        .send()
        .await
        .unwrap();

    let row = sqlx::query!(
        "SELECT encrypted_token FROM service_connections WHERE user_id = $1 AND service = 'github'",
        alice_id,
    )
    .fetch_one(&app.pool)
    .await
    .unwrap();

    let stored: Vec<u8> = row.encrypted_token;

    let plaintext_bytes = secret_token.as_bytes();
    assert!(
        !stored
            .windows(plaintext_bytes.len())
            .any(|w| w == plaintext_bytes),
        "DB blob should not contain plaintext token bytes"
    );

    assert!(
        stored.len() >= 28,
        "Stored blob suspiciously short: {} bytes",
        stored.len()
    );
}

#[tokio::test]
async fn same_token_encrypts_to_different_ciphertexts() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let bob = register_and_login(&app.address, "bob@example.com").await;
    let alice_id = get_user_id(&app.address, &alice).await;
    let bob_id = get_user_id(&app.address, &bob).await;

    let same_token = "ghp_SAME_TOKEN_FOR_BOTH_USERS";
    let client = reqwest::Client::new();

    for token in [&alice, &bob] {
        client
            .post(format!("{}/me/services/github", app.address))
            .bearer_auth(token)
            .json(&json!({ "token": same_token }))
            .send()
            .await
            .unwrap();
    }

    let alice_row = sqlx::query!(
        "SELECT encrypted_token FROM service_connections WHERE user_id = $1",
        alice_id,
    )
    .fetch_one(&app.pool)
    .await
    .unwrap();
    let bob_row = sqlx::query!(
        "SELECT encrypted_token FROM service_connections WHERE user_id = $1",
        bob_id,
    )
    .fetch_one(&app.pool)
    .await
    .unwrap();

    assert_ne!(
        alice_row.encrypted_token, bob_row.encrypted_token,
        "Random nonce must make identical plaintexts encrypt to different ciphertexts"
    );
}

#[tokio::test]
async fn reconnecting_service_overwrites_previous_token() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let alice_id = get_user_id(&app.address, &alice).await;

    let client = reqwest::Client::new();

    client
        .post(format!("{}/me/services/github", app.address))
        .bearer_auth(&alice)
        .json(&json!({ "token": "old_token_v1" }))
        .send()
        .await
        .unwrap();

    let first_blob = sqlx::query!(
        "SELECT encrypted_token FROM service_connections WHERE user_id = $1",
        alice_id,
    )
    .fetch_one(&app.pool)
    .await
    .unwrap()
    .encrypted_token;

    client
        .post(format!("{}/me/services/github", app.address))
        .bearer_auth(&alice)
        .json(&json!({ "token": "new_token_v2" }))
        .send()
        .await
        .unwrap();

    let second_blob = sqlx::query!(
        "SELECT encrypted_token FROM service_connections WHERE user_id = $1",
        alice_id,
    )
    .fetch_one(&app.pool)
    .await
    .unwrap()
    .encrypted_token;

    let count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM service_connections WHERE user_id = $1")
            .bind(alice_id)
            .fetch_one(&app.pool)
            .await
            .unwrap();
    assert_eq!(count.0, 1);

    assert_ne!(first_blob, second_blob);
}

#[tokio::test]
async fn list_returns_connected_services_without_tokens() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let secret_token = "ghp_LIST_TEST_TOKEN_XYZ";

    let client = reqwest::Client::new();
    client
        .post(format!("{}/me/services/github", app.address))
        .bearer_auth(&alice)
        .json(&json!({ "token": secret_token }))
        .send()
        .await
        .unwrap();
    client
        .post(format!("{}/me/services/discord", app.address))
        .bearer_auth(&alice)
        .json(&json!({ "token": "https://discord.com/webhook/whatever" }))
        .send()
        .await
        .unwrap();

    let res = client
        .get(format!("{}/me/services", app.address))
        .bearer_auth(&alice)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);

    let body_text = res.text().await.unwrap();
    assert!(
        !body_text.contains(secret_token),
        "List response leaked token: {body_text}"
    );

    let list: Vec<serde_json::Value> = serde_json::from_str(&body_text).unwrap();
    assert_eq!(list.len(), 2);
    let services: Vec<&str> = list
        .iter()
        .map(|v| v["service"].as_str().unwrap())
        .collect();
    assert!(services.contains(&"github"));
    assert!(services.contains(&"discord"));
}

#[tokio::test]
async fn users_cannot_see_each_others_connections() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let bob = register_and_login(&app.address, "bob@example.com").await;

    let client = reqwest::Client::new();
    client
        .post(format!("{}/me/services/github", app.address))
        .bearer_auth(&alice)
        .json(&json!({ "token": "alice_token" }))
        .send()
        .await
        .unwrap();

    let res = client
        .get(format!("{}/me/services", app.address))
        .bearer_auth(&bob)
        .send()
        .await
        .unwrap();

    let list: Vec<serde_json::Value> = res.json().await.unwrap();
    assert_eq!(list.len(), 0);
}

#[tokio::test]
async fn disconnect_removes_the_connection() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;

    let client = reqwest::Client::new();
    client
        .post(format!("{}/me/services/github", app.address))
        .bearer_auth(&alice)
        .json(&json!({ "token": "some_token" }))
        .send()
        .await
        .unwrap();

    let del_res = client
        .delete(format!("{}/me/services/github", app.address))
        .bearer_auth(&alice)
        .send()
        .await
        .unwrap();
    assert_eq!(del_res.status(), 204);

    let list_res = client
        .get(format!("{}/me/services", app.address))
        .bearer_auth(&alice)
        .send()
        .await
        .unwrap();
    let list: Vec<serde_json::Value> = list_res.json().await.unwrap();
    assert_eq!(list.len(), 0);
}

#[tokio::test]
async fn disconnect_nonexistent_service_returns_404() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;

    let client = reqwest::Client::new();
    let res = client
        .delete(format!("{}/me/services/github", app.address))
        .bearer_auth(&alice)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 404);
}

#[tokio::test]
async fn unknown_service_returns_404() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;

    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/me/services/slack", app.address))
        .bearer_auth(&alice)
        .json(&json!({ "token": "whatever" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 404);
}

#[tokio::test]
async fn empty_token_returns_422() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;

    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/me/services/github", app.address))
        .bearer_auth(&alice)
        .json(&json!({ "token": "   " })) // whitespace only
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 422);
}

#[tokio::test]
async fn connect_requires_authentication() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/me/services/github", app.address))
        .json(&json!({ "token": "whatever" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 401);
}
