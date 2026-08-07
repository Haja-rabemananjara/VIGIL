use reqwest::StatusCode;
use serde_json::json;

mod common;
use common::spawn_app;

async fn authenticate(address: &str, client: &reqwest::Client) -> String {
    let email = format!("user-{}@example.com", uuid::Uuid::new_v4());

    client
        .post(format!("{address}/auth/signup"))
        .json(&json!({
            "email": email, "password": "validpassword", "display_name": "Tester"
        }))
        .send()
        .await
        .unwrap();

    let res = client
        .post(format!("{address}/auth/signin"))
        .json(&json!({ "email": email, "password": "validpassword" }))
        .send()
        .await
        .unwrap();

    let body: serde_json::Value = res.json().await.unwrap();
    body["token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn update_display_name() {
    let app = spawn_app().await;
    let token = authenticate(&app.address, &app.client).await;

    let res = app
        .client
        .patch(format!("{}/me", app.address))
        .bearer_auth(&token)
        .json(&json!({ "display_name": "New Name" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["display_name"], "New Name");

    let res = app
        .client
        .get(format!("{}/me", app.address))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["display_name"], "New Name");
}

#[tokio::test]
async fn update_language() {
    let app = spawn_app().await;
    let token = authenticate(&app.address, &app.client).await;

    let res = app
        .client
        .patch(format!("{}/me", app.address))
        .bearer_auth(&token)
        .json(&json!({ "language": "fr" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["language"], "fr");

    let res = app
        .client
        .get(format!("{}/me", app.address))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["language"], "fr");
}

#[tokio::test]
async fn update_password() {
    let app = spawn_app().await;
    let email = format!("user-{}@example.com", uuid::Uuid::new_v4());

    app.client
        .post(format!("{}/auth/signup", app.address))
        .json(&json!({
            "email": email, "password": "oldpassword1", "display_name": "Tester"
        }))
        .send()
        .await
        .unwrap();

    let res = app
        .client
        .post(format!("{}/auth/signin", app.address))
        .json(&json!({ "email": email, "password": "oldpassword1" }))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = res.json().await.unwrap();
    let token = body["token"].as_str().unwrap();

    let res = app
        .client
        .patch(format!("{}/me", app.address))
        .bearer_auth(token)
        .json(&json!({ "password": "newpassword1" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body.get("password_hash").is_none());

    let res = app
        .client
        .post(format!("{}/auth/signin", app.address))
        .json(&json!({ "email": email, "password": "oldpassword1" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    let res = app
        .client
        .post(format!("{}/auth/signin", app.address))
        .json(&json!({ "email": email, "password": "newpassword1" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn update_multiple_fields() {
    let app = spawn_app().await;
    let token = authenticate(&app.address, &app.client).await;

    let res = app
        .client
        .patch(format!("{}/me", app.address))
        .bearer_auth(&token)
        .json(&json!({
            "display_name": "Updated",
            "language": "fr"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["display_name"], "Updated");
    assert_eq!(body["language"], "fr");
}

#[tokio::test]
async fn empty_display_name_rejected() {
    let app = spawn_app().await;
    let token = authenticate(&app.address, &app.client).await;

    let res = app
        .client
        .patch(format!("{}/me", app.address))
        .bearer_auth(&token)
        .json(&json!({ "display_name": "   " }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn invalid_language_rejected() {
    let app = spawn_app().await;
    let token = authenticate(&app.address, &app.client).await;

    let res = app
        .client
        .patch(format!("{}/me", app.address))
        .bearer_auth(&token)
        .json(&json!({ "language": "de" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn short_password_rejected() {
    let app = spawn_app().await;
    let token = authenticate(&app.address, &app.client).await;

    let res = app
        .client
        .patch(format!("{}/me", app.address))
        .bearer_auth(&token)
        .json(&json!({ "password": "short" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn unauthenticated_patch_rejected() {
    let app = spawn_app().await;

    let res = app
        .client
        .patch(format!("{}/me", app.address))
        .json(&json!({ "display_name": "Hacker" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}
