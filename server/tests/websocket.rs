use futures_util::StreamExt;
use serde_json::json;
use tokio_tungstenite::{connect_async, tungstenite::Message};

mod common;
use common::spawn_app;

async fn register_and_login(address: &str, email: &str) -> String {
    let client = reqwest::Client::new();
    client
        .post(format!("{address}/auth/signup"))
        .json(&json!({ "email": email, "password": "password123", "display_name": "Test" }))
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

fn ws_url(http_address: &str, token: &str) -> String {
    let base = http_address.replace("http://", "ws://");
    format!("{base}/ws?token={token}")
}

#[tokio::test]
async fn ws_connection_with_valid_token_succeeds() {
    let app = spawn_app().await;
    let token = register_and_login(&app.address, "alice@example.com").await;

    let url = ws_url(&app.address, &token);
    let (mut ws_stream, _) = connect_async(&url).await.expect("ws connect failed");

    let msg = ws_stream.next().await.unwrap().unwrap();
    let text = match msg {
        Message::Text(t) => t.to_string(),
        _ => panic!("expected text message"),
    };
    let event: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(event["type"], "connected");
    assert!(event["user_id"].as_str().is_some());

    ws_stream.close(None).await.unwrap();
}

#[tokio::test]
async fn ws_connection_with_invalid_token_is_rejected() {
    let app = spawn_app().await;

    let url = ws_url(&app.address, "fake-token-that-does-not-exist");
    let result = connect_async(&url).await;
    assert!(result.is_err(), "expected connection to fail");
}

#[tokio::test]
async fn ws_connection_with_missing_token_is_rejected() {
    let app = spawn_app().await;

    let ws_base = app.address.replace("http://", "ws://");
    let url = format!("{ws_base}/ws");

    let result = connect_async(&url).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn multiple_tabs_per_user_all_receive_events() {
    let app = spawn_app().await;
    let token = register_and_login(&app.address, "alice@example.com").await;

    let url = ws_url(&app.address, &token);
    let (mut tab1, _) = connect_async(&url).await.unwrap();
    let (mut tab2, _) = connect_async(&url).await.unwrap();

    let _hello1 = tab1.next().await.unwrap().unwrap();
    let _hello2 = tab2.next().await.unwrap().unwrap();

    tab1.close(None).await.unwrap();
    tab2.close(None).await.unwrap();
}
