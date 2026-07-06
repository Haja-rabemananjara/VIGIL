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

async fn create_invitation(address: &str, token: &str, team_id: Uuid) -> String {
    let client = reqwest::Client::new();
    let res = client
        .post(format!("{address}/teams/{team_id}/invitations"))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = res.json().await.unwrap();
    body["code"].as_str().unwrap().to_string()
}

async fn join_team(address: &str, token: &str, code: &str) {
    let client = reqwest::Client::new();
    client
        .post(format!("{address}/teams/join"))
        .bearer_auth(token)
        .json(&json!({ "code": code }))
        .send()
        .await
        .unwrap();
}

fn valid_rule_payload() -> serde_json::Value {
    json!({
        "name": "CI failure → critical incident",
        "enabled": true,
        "trigger": {
            "service": "github",
            "event": "workflow_run",
            "filters": { "conclusion": "failure" }
        },
        "reaction": {
            "type": "vigil_create_incident",
            "payload": {
                "title": "CI broken on {{repository.name}}",
                "severity": "high"
            }
        }
    })
}

#[tokio::test]
async fn manager_creates_rule_returns_201() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops").await;

    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/teams/{team_id}/rules", app.address))
        .bearer_auth(&alice)
        .json(&valid_rule_payload())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 201);

    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["name"], "CI failure → critical incident");
    assert_eq!(body["enabled"], true);
    assert_eq!(body["trigger_service"], "github");
    assert_eq!(body["trigger_event"], "workflow_run");
    assert_eq!(body["reaction_type"], "vigil_create_incident");
    assert!(Uuid::parse_str(body["id"].as_str().unwrap()).is_ok());
}

#[tokio::test]
async fn observer_cannot_create_rule_returns_403() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let bob = register_and_login(&app.address, "bob@example.com").await;

    let team_id = create_team(&app.address, &alice, "Ops").await;
    let code = create_invitation(&app.address, &alice, team_id).await;
    join_team(&app.address, &bob, &code).await; // Bob joins as Observer

    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/teams/{team_id}/rules", app.address))
        .bearer_auth(&bob)
        .json(&valid_rule_payload())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 403);
}

#[tokio::test]
async fn non_member_cannot_create_rule_returns_404() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let charlie = register_and_login(&app.address, "charlie@example.com").await;

    let team_id = create_team(&app.address, &alice, "Ops").await;

    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/teams/{team_id}/rules", app.address))
        .bearer_auth(&charlie) // Not a member — should get 404, not 403
        .json(&valid_rule_payload())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 404);
}

#[tokio::test]
async fn unknown_trigger_returns_422() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops").await;

    let mut payload = valid_rule_payload();
    payload["trigger"]["service"] = json!("gitlab_but_typoed");

    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/teams/{team_id}/rules", app.address))
        .bearer_auth(&alice)
        .json(&payload)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 422);
}

#[tokio::test]
async fn unknown_reaction_returns_422() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops").await;

    let mut payload = valid_rule_payload();
    payload["reaction"]["type"] = json!("send_to_slack_maybe");

    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/teams/{team_id}/rules", app.address))
        .bearer_auth(&alice)
        .json(&payload)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 422);
}

#[tokio::test]
async fn empty_name_returns_422() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops").await;

    let mut payload = valid_rule_payload();
    payload["name"] = json!("   ");

    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/teams/{team_id}/rules", app.address))
        .bearer_auth(&alice)
        .json(&payload)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 422);
}

#[tokio::test]
async fn list_returns_created_rules() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops").await;

    let client = reqwest::Client::new();
    client
        .post(format!("{}/teams/{team_id}/rules", app.address))
        .bearer_auth(&alice)
        .json(&valid_rule_payload())
        .send()
        .await
        .unwrap();

    let res = client
        .get(format!("{}/teams/{team_id}/rules", app.address))
        .bearer_auth(&alice)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);

    let body: serde_json::Value = res.json().await.unwrap();
    let arr = body.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["name"], "CI failure → critical incident");
}

#[tokio::test]
async fn observer_can_list_rules() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let bob = register_and_login(&app.address, "bob@example.com").await;

    let team_id = create_team(&app.address, &alice, "Ops").await;
    let code = create_invitation(&app.address, &alice, team_id).await;
    join_team(&app.address, &bob, &code).await;

    let client = reqwest::Client::new();
    let res = client
        .get(format!("{}/teams/{team_id}/rules", app.address))
        .bearer_auth(&bob)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
}

#[tokio::test]
async fn get_nonexistent_rule_returns_404() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops").await;
    let fake_rule_id = Uuid::new_v4();

    let client = reqwest::Client::new();
    let res = client
        .get(format!(
            "{}/teams/{team_id}/rules/{fake_rule_id}",
            app.address
        ))
        .bearer_auth(&alice)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 404);
}

#[tokio::test]
async fn manager_can_toggle_enabled() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops").await;

    let client = reqwest::Client::new();
    let create_res = client
        .post(format!("{}/teams/{team_id}/rules", app.address))
        .bearer_auth(&alice)
        .json(&valid_rule_payload())
        .send()
        .await
        .unwrap();
    let created: serde_json::Value = create_res.json().await.unwrap();
    let rule_id = created["id"].as_str().unwrap();

    let patch_res = client
        .patch(format!("{}/teams/{team_id}/rules/{rule_id}", app.address))
        .bearer_auth(&alice)
        .json(&json!({ "enabled": false }))
        .send()
        .await
        .unwrap();

    assert_eq!(patch_res.status(), 200);
    let updated: serde_json::Value = patch_res.json().await.unwrap();
    assert_eq!(updated["enabled"], false);
    assert_eq!(updated["name"], "CI failure → critical incident");
}

#[tokio::test]
async fn observer_cannot_patch_rule() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let bob = register_and_login(&app.address, "bob@example.com").await;

    let team_id = create_team(&app.address, &alice, "Ops").await;
    let code = create_invitation(&app.address, &alice, team_id).await;
    join_team(&app.address, &bob, &code).await;

    let client = reqwest::Client::new();
    let create_res = client
        .post(format!("{}/teams/{team_id}/rules", app.address))
        .bearer_auth(&alice)
        .json(&valid_rule_payload())
        .send()
        .await
        .unwrap();
    let created: serde_json::Value = create_res.json().await.unwrap();
    let rule_id = created["id"].as_str().unwrap();

    let patch_res = client
        .patch(format!("{}/teams/{team_id}/rules/{rule_id}", app.address))
        .bearer_auth(&bob)
        .json(&json!({ "enabled": false }))
        .send()
        .await
        .unwrap();

    assert_eq!(patch_res.status(), 403);
}

#[tokio::test]
async fn manager_can_delete_rule() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops").await;

    let client = reqwest::Client::new();
    let create_res = client
        .post(format!("{}/teams/{team_id}/rules", app.address))
        .bearer_auth(&alice)
        .json(&valid_rule_payload())
        .send()
        .await
        .unwrap();
    let created: serde_json::Value = create_res.json().await.unwrap();
    let rule_id = created["id"].as_str().unwrap();

    let delete_res = client
        .delete(format!("{}/teams/{team_id}/rules/{rule_id}", app.address))
        .bearer_auth(&alice)
        .send()
        .await
        .unwrap();

    assert_eq!(delete_res.status(), 204);

    let get_res = client
        .get(format!("{}/teams/{team_id}/rules/{rule_id}", app.address))
        .bearer_auth(&alice)
        .send()
        .await
        .unwrap();
    assert_eq!(get_res.status(), 404);
}

#[tokio::test]
async fn delete_nonexistent_rule_returns_404() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops").await;
    let fake_rule_id = Uuid::new_v4();

    let client = reqwest::Client::new();
    let res = client
        .delete(format!(
            "{}/teams/{team_id}/rules/{fake_rule_id}",
            app.address
        ))
        .bearer_auth(&alice)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 404);
}

#[tokio::test]
async fn filters_and_payload_roundtrip_as_json() {
    let app = spawn_app().await;
    let alice = register_and_login(&app.address, "alice@example.com").await;
    let team_id = create_team(&app.address, &alice, "Ops").await;

    let client = reqwest::Client::new();
    let create_res = client
        .post(format!("{}/teams/{team_id}/rules", app.address))
        .bearer_auth(&alice)
        .json(&valid_rule_payload())
        .send()
        .await
        .unwrap();
    let created: serde_json::Value = create_res.json().await.unwrap();

    assert_eq!(created["trigger_filters"]["conclusion"], "failure");
    assert_eq!(created["reaction_payload"]["severity"], "high");
    assert_eq!(
        created["reaction_payload"]["title"],
        "CI broken on {{repository.name}}"
    );
}
