mod common;
use common::spawn_app;

async fn create_user(address: &str, client: &reqwest::Client) -> String {
    let email = format!("user-{}@example.com", uuid::Uuid::new_v4());
    client
        .post(format!("{address}/auth/signup"))
        .json(&serde_json::json!({
            "email": email,
            "password": "validpassword",
            "display_name": "Test User"
        }))
        .send()
        .await
        .unwrap();
    email
}

#[tokio::test]
async fn signin_returns_token_and_user() {
    let app = spawn_app().await;
    let email = create_user(&app.address, &app.client).await;

    let res = app
        .client
        .post(format!("{}/auth/signin", app.address))
        .json(&serde_json::json!({ "email": email, "password": "validpassword" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status().as_u16(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(
        body["token"].as_str().unwrap().len() == 64,
        "32 bytes = 64 hex chars"
    );
    assert_eq!(body["user"]["email"], email);
    assert!(body["user"].get("password_hash").is_none());
}

#[tokio::test]
async fn signin_wrong_password_returns_401() {
    let app = spawn_app().await;
    let email = create_user(&app.address, &app.client).await;

    let res = app
        .client
        .post(format!("{}/auth/signin", app.address))
        .json(&serde_json::json!({ "email": email, "password": "wrongpassword" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status().as_u16(), 401);
}

#[tokio::test]
async fn signin_unknown_email_returns_401() {
    let app = spawn_app().await;

    let res = app
        .client
        .post(format!("{}/auth/signin", app.address))
        .json(&serde_json::json!({ "email": "nobody@example.com", "password": "whatever1" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status().as_u16(), 401);
}

#[tokio::test]
async fn signin_expired_token_is_rejected() {}
