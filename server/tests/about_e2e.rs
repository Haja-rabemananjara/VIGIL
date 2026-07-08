mod common;
use common::spawn_app;

#[tokio::test]
async fn about_returns_expected_top_level_shape() {
    let app = spawn_app().await;

    let res = reqwest::Client::new()
        .get(format!("{}/about.json", app.address))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);

    let body: serde_json::Value = res.json().await.unwrap();

    assert!(body["client"].is_object());
    assert!(body["client"]["host"].is_string());
    assert!(body["server"].is_object());
    assert!(body["server"]["current_time"].is_number());
    assert!(body["server"]["services"].is_array());
    assert!(body["server"]["token"].is_string());
}

#[tokio::test]
async fn about_exposes_correct_kickoff_token() {
    let app = spawn_app().await;

    let expected = server::config::compute_kickoff_token("test_first", "test_login");

    let res = reqwest::Client::new()
        .get(format!("{}/about.json", app.address))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = res.json().await.unwrap();

    assert_eq!(body["server"]["token"].as_str().unwrap(), expected);
}

#[tokio::test]
async fn about_lists_registered_github_actions() {
    let app = spawn_app().await;

    let res = reqwest::Client::new()
        .get(format!("{}/about.json", app.address))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = res.json().await.unwrap();

    let services = body["server"]["services"].as_array().unwrap();

    let github = services
        .iter()
        .find(|s| s["name"] == "github")
        .expect("github service should be listed");

    let actions = github["actions"].as_array().unwrap();
    let event_names: Vec<&str> = actions
        .iter()
        .map(|a| a["name"].as_str().unwrap())
        .collect();

    assert!(event_names.contains(&"workflow_run"));
    assert!(event_names.contains(&"push"));
    assert!(event_names.contains(&"pull_request"));
}

#[tokio::test]
async fn about_lists_registered_vigil_and_discord_reactions() {
    let app = spawn_app().await;

    let res = reqwest::Client::new()
        .get(format!("{}/about.json", app.address))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = res.json().await.unwrap();

    let services = body["server"]["services"].as_array().unwrap();

    let vigil = services
        .iter()
        .find(|s| s["name"] == "vigil")
        .expect("vigil service should be listed");
    let vigil_reactions = vigil["reactions"].as_array().unwrap();
    let vigil_kinds: Vec<&str> = vigil_reactions
        .iter()
        .map(|r| r["name"].as_str().unwrap())
        .collect();
    assert!(vigil_kinds.contains(&"vigil_create_incident"));
    assert!(vigil_kinds.contains(&"vigil_escalate_incident"));
    assert!(vigil_kinds.contains(&"vigil_block_release"));
    assert!(vigil_kinds.contains(&"vigil_validate_release_step"));

    let discord = services
        .iter()
        .find(|s| s["name"] == "discord")
        .expect("discord service should be listed");
    let discord_reactions = discord["reactions"].as_array().unwrap();
    let discord_kinds: Vec<&str> = discord_reactions
        .iter()
        .map(|r| r["name"].as_str().unwrap())
        .collect();
    assert!(discord_kinds.contains(&"discord_message"));
}

#[tokio::test]
async fn about_is_public_no_auth_required() {
    let app = spawn_app().await;

    let res = reqwest::Client::new()
        .get(format!("{}/about.json", app.address))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
}
