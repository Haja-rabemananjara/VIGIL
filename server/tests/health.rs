mod common;

use common::spawn_app;

#[tokio::test]
async fn health_returns_200_and_version() {
    let app = spawn_app().await;

    let response = app
        .client
        .get(format!("{}/health", app.address))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["status"], "ok");
    assert!(body["version"].is_string());

    app.cleanup().await;
}

#[tokio::test]
async fn unknown_route_returns_404() {
    let app = spawn_app().await;

    let response = app
        .client
        .get(format!("{}/nonexistent", app.address))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 404);

    app.cleanup().await;
}
