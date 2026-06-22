mod common;

use common::spawn_app;

#[tokio::test]
async fn signup_returns_201_and_never_exposes_the_hash() {
    let app = spawn_app().await;

    let response = app
        .client
        .post(format!("{}/auth/signup", app.address))
        .json(&serde_json::json!({
            "email": "alice@example.com",
            "password": "correct horse battery",
            "display_name": "Alice"
        }))
        .send()
        .await
        .expect("request failed");

    assert_eq!(response.status().as_u16(), 201);
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["email"], "alice@example.com");
    assert_eq!(body["display_name"], "Alice");
    assert!(body.get("id").is_some());
    assert!(body.get("password_hash").is_none(), "hash must never leak");
}

#[tokio::test]
async fn signup_rejects_duplicate_email_case_insensitive_with_409() {
    let app = spawn_app().await;
    let client = app.client;
    let url = format!("{}/auth/signup", app.address);

    let first = client
        .post(&url)
        .json(&serde_json::json!({
            "email": "bob@example.com", "password": "longenough1", "display_name": "Bob"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(first.status().as_u16(), 201);

    // Different casing → must still collide.
    let second = client
        .post(&url)
        .json(&serde_json::json!({
            "email": "BOB@Example.com", "password": "longenough1", "display_name": "Bobby"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(second.status().as_u16(), 409);
}

#[tokio::test]
async fn signup_rejects_short_password_with_422() {
    let app = spawn_app().await;
    let client = app.client;

    let response = client
        .post(format!("{}/auth/signup", app.address))
        .json(&serde_json::json!({
            "email": "carol@example.com", "password": "short", "display_name": "Carol"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 422);
}
