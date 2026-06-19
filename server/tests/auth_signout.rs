mod common;
use common::spawn_app;

async fn authenticate(address: &str, client: &reqwest::Client) -> String {
    let email = format!("user-{}@example.com", uuid::Uuid::new_v4());
    client
        .post(format!("{address}/auth/signup"))
        .json(&serde_json::json!({
            "email": email, "password": "validpassword", "display_name": "Tester"
        }))
        .send()
        .await
        .unwrap();
    let res = client
        .post(format!("{address}/auth/signin"))
        .json(&serde_json::json!({ "email": email, "password": "validpassword" }))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = res.json().await.unwrap();
    body["token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn signout_returns_204_and_invalidates_token() {
    let app = spawn_app().await;
    let token = authenticate(&app.address, &app.client).await;
    let auth_header = format!("Bearer {token}");

    let res = app
        .client
        .get(format!("{}/me", app.address))
        .header("Authorization", &auth_header)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 200);

    let res = app
        .client
        .post(format!("{}/auth/signout", app.address))
        .header("Authorization", &auth_header)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 204);

    let res = app
        .client
        .get(format!("{}/me", app.address))
        .header("Authorization", &auth_header)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 401);
}

#[tokio::test]
async fn signout_without_token_returns_401() {
    let app = spawn_app().await;

    let res = app
        .client
        .post(format!("{}/auth/signout", app.address))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status().as_u16(), 401);
}
