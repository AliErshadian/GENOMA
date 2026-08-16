use std::path::PathBuf;
use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use genoma_api::config::AppConfig;
use genoma_api::{app, state::AppState};
use serde_json::Value;
use tower::ServiceExt;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn ensure_demo_evolve_repo() {
    let repo = workspace_root().join("data/repos/demo-evolve");
    if repo.join(".git").is_dir() || repo.join("HEAD").is_file() {
        return;
    }
    let script = workspace_root().join("scripts/seed-demo-evolve.sh");
    let status = std::process::Command::new("bash")
        .arg(&script)
        .status()
        .expect("run seed-demo-evolve.sh");
    assert!(status.success(), "failed to seed demo-evolve repo");
}

async fn test_app() -> axum::Router {
    ensure_demo_evolve_repo();
    let mut config = AppConfig::for_tests(workspace_root());
    config.blob_dir = std::env::temp_dir().join(format!("genoma-test-{}", uuid::Uuid::new_v4()));
    let state = AppState::with_backends(config, None, None)
        .await
        .expect("test state");
    app(state)
}

async fn json_request(app: &axum::Router, method: &str, uri: &str) -> (StatusCode, Value) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, value)
}

#[tokio::test]
async fn demo_analysis_completes_with_real_progress() {
    let app = test_app().await;
    let (status, created) = json_request(&app, "POST", "/api/v1/analyses/demo?file=sample.txt").await;
    assert_eq!(status, StatusCode::OK);
    let id = created["id"].as_str().expect("analysis id");

    let mut dna = Value::Null;
    let mut latest = Value::Null;
    for _ in 0..80 {
        tokio::time::sleep(Duration::from_millis(50)).await;
        let (progress_status, progress) =
            json_request(&app, "GET", &format!("/api/v1/analyses/{id}/progress/latest")).await;
        assert_eq!(progress_status, StatusCode::OK);
        latest = progress;
        assert!(latest["processed_bytes"].as_u64().is_some());
        assert!(latest["stage"].as_str().is_some());

        let (get_status, body) = json_request(&app, "GET", &format!("/api/v1/analyses/{id}")).await;
        assert_eq!(get_status, StatusCode::OK);
        if body["status"] == "COMPLETE" {
            let (dna_status, fingerprint) = json_request(&app, "GET", &format!("/api/v1/dna/{id}")).await;
            assert_eq!(dna_status, StatusCode::OK);
            dna = fingerprint;
            break;
        }
        if body["status"] == "FAILED" {
            panic!("analysis failed: {body}");
        }
    }

    assert_eq!(dna["generator_version"], "dna-v1");
    assert!(dna["raw"]["entropy"].as_f64().is_some());
    assert_eq!(latest["stage"], "COMPLETE");
    assert!(latest["processed_bytes"].as_u64().unwrap() > 0);

    let (list_status, list) = json_request(&app, "GET", "/api/v1/analyses").await;
    assert_eq!(list_status, StatusCode::OK);
    assert!(list.as_array().unwrap().iter().any(|item| item["id"] == id));
}

#[tokio::test]
async fn health_reports_generator() {
    let app = test_app().await;
    let (status, body) = json_request(&app, "GET", "/api/v1/health").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["name"], "GENOMA");
    assert_eq!(body["generator"], "dna-v1");
}

async fn wait_complete(app: &axum::Router, id: &str) -> Value {
    for _ in 0..80 {
        tokio::time::sleep(Duration::from_millis(50)).await;
        let (status, body) = json_request(app, "GET", &format!("/api/v1/analyses/{id}")).await;
        assert_eq!(status, StatusCode::OK);
        if body["status"] == "COMPLETE" {
            return body;
        }
        if body["status"] == "FAILED" {
            panic!("analysis failed: {body}");
        }
    }
    panic!("analysis did not complete: {id}");
}

async fn post_json(app: &axum::Router, uri: &str, body: Value) -> (StatusCode, Value) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(uri)
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, value)
}

#[tokio::test]
async fn compare_returns_similarity_for_completed_analyses() {
    let app = test_app().await;
    let (_, left) = json_request(&app, "POST", "/api/v1/analyses/demo?file=sample.txt").await;
    let (_, right) = json_request(&app, "POST", "/api/v1/analyses/demo?file=sample.bin").await;
    let left_id = left["id"].as_str().expect("left id");
    let right_id = right["id"].as_str().expect("right id");
    wait_complete(&app, left_id).await;
    wait_complete(&app, right_id).await;

    let (status, body) = post_json(
        &app,
        "/api/v1/compare",
        serde_json::json!({ "left_id": left_id, "right_id": right_id }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["left_id"], left_id);
    assert_eq!(body["right_id"], right_id);
    assert!(body["similarity"]["overall"].as_f64().unwrap() > 0.0);
    assert!(body["similarity"]["entropy"].as_f64().is_some());

    let (same_status, same) = post_json(
        &app,
        "/api/v1/compare",
        serde_json::json!({ "left_id": left_id, "right_id": left_id }),
    )
    .await;
    assert_eq!(same_status, StatusCode::OK);
    assert!(same["similarity"]["overall"].as_f64().unwrap() > 0.99);

    let missing = uuid::Uuid::new_v4().to_string();
    let (missing_status, _) = post_json(
        &app,
        "/api/v1/compare",
        serde_json::json!({ "left_id": missing, "right_id": right_id }),
    )
    .await;
    assert_eq!(missing_status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn mutations_detect_chunk_diffs_between_analyses() {
    let app = test_app().await;
    let (_, baseline) = json_request(&app, "POST", "/api/v1/analyses/demo?file=sample.txt").await;
    let (_, current) = json_request(&app, "POST", "/api/v1/analyses/demo?file=sample.bin").await;
    let baseline_id = baseline["id"].as_str().expect("baseline id");
    let current_id = current["id"].as_str().expect("current id");
    wait_complete(&app, baseline_id).await;
    wait_complete(&app, current_id).await;

    let (status, body) = post_json(
        &app,
        "/api/v1/mutations",
        serde_json::json!({ "baseline_id": baseline_id, "current_id": current_id }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["baseline_id"], baseline_id);
    assert_eq!(body["current_id"], current_id);
    let mutations = body["mutations"].as_array().expect("mutations array");
    assert!(!mutations.is_empty());
    assert!(mutations[0]["impact"].as_f64().is_some());

    let (same_status, same) = post_json(
        &app,
        "/api/v1/mutations",
        serde_json::json!({ "baseline_id": baseline_id, "current_id": baseline_id }),
    )
    .await;
    assert_eq!(same_status, StatusCode::OK);
    assert!(same["mutations"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn galaxy_returns_nodes_for_completed_analyses() {
    let app = test_app().await;
    let (_, a) = json_request(&app, "POST", "/api/v1/analyses/demo?file=sample.txt").await;
    let (_, b) = json_request(&app, "POST", "/api/v1/analyses/demo?file=sample.bin").await;
    let a_id = a["id"].as_str().expect("a id");
    let b_id = b["id"].as_str().expect("b id");
    wait_complete(&app, a_id).await;
    wait_complete(&app, b_id).await;

    let (status, body) = post_json(
        &app,
        "/api/v1/galaxy",
        serde_json::json!({ "analysis_ids": [a_id, b_id, a_id] }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let nodes = body["nodes"].as_array().expect("nodes");
    assert_eq!(nodes.len(), 2);
    assert!(nodes.iter().any(|node| node["id"] == a_id));
    assert!(nodes.iter().any(|node| node["id"] == b_id));
    assert!(nodes[0]["entropy"].as_f64().is_some());
    assert!(nodes[0]["generator_version"].as_str().unwrap() == "dna-v1");
    assert!(nodes[0]["cluster_id"].as_u64().is_some());
    assert!(body["cluster_count"].as_u64().unwrap() >= 1);
    assert_eq!(nodes[0]["position"].as_array().unwrap().len(), 3);
    assert!(body["links"].as_array().is_some());

    let (empty_status, _) = post_json(
        &app,
        "/api/v1/galaxy",
        serde_json::json!({ "analysis_ids": [] }),
    )
    .await;
    assert_eq!(empty_status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn evolution_series_round_trip() {
    let app = test_app().await;
    let (_, a) = json_request(&app, "POST", "/api/v1/analyses/demo?file=sample.txt").await;
    let (_, b) = json_request(&app, "POST", "/api/v1/analyses/demo?file=sample.bin").await;
    let a_id = a["id"].as_str().expect("a id");
    let b_id = b["id"].as_str().expect("b id");
    wait_complete(&app, a_id).await;
    wait_complete(&app, b_id).await;

    let (status, created) = post_json(
        &app,
        "/api/v1/evolution",
        serde_json::json!({
            "name": "demo-series",
            "snapshots": [
                { "analysis_id": a_id, "version_label": "v1" },
                { "analysis_id": b_id, "version_label": "v2" }
            ]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(created["name"], "demo-series");
    let series_id = created["id"].as_str().expect("series id");
    assert_eq!(created["snapshots"].as_array().unwrap().len(), 2);

    let (get_status, fetched) =
        json_request(&app, "GET", &format!("/api/v1/evolution/{series_id}")).await;
    assert_eq!(get_status, StatusCode::OK);
    assert_eq!(fetched["id"], series_id);
    assert_eq!(fetched["snapshots"][0]["version_label"], "v1");
    assert_eq!(fetched["snapshots"][1]["analysis_id"], b_id);

    let (list_status, list) = json_request(&app, "GET", "/api/v1/evolution").await;
    assert_eq!(list_status, StatusCode::OK);
    assert!(list
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["id"] == series_id));
}

#[tokio::test]
async fn evolution_git_import_builds_series_from_demo_repo() {
    let app = test_app().await;
    let (status, body) = post_json(
        &app,
        "/api/v1/evolution/git",
        serde_json::json!({
            "repo": "demo-evolve",
            "path": "sample.txt",
            "max_commits": 3
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let snapshots = body["snapshots"].as_array().expect("snapshots");
    assert!(snapshots.len() >= 3);
    assert!(body["name"].as_str().unwrap().contains("demo-evolve"));

    let (bad_status, _) = post_json(
        &app,
        "/api/v1/evolution/git",
        serde_json::json!({ "repo": "../etc", "path": "sample.txt" }),
    )
    .await;
    assert_eq!(bad_status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn experiments_isolation_and_knn() {
    let app = test_app().await;
    let (_, a) = json_request(&app, "POST", "/api/v1/analyses/demo?file=sample.txt").await;
    let (_, b) = json_request(&app, "POST", "/api/v1/analyses/demo?file=sample.bin").await;
    let a_id = a["id"].as_str().expect("a id");
    let b_id = b["id"].as_str().expect("b id");
    wait_complete(&app, a_id).await;
    wait_complete(&app, b_id).await;

    let (iso_status, iso) = post_json(
        &app,
        "/api/v1/experiments/isolation",
        serde_json::json!({ "analysis_id": a_id }),
    )
    .await;
    assert_eq!(iso_status, StatusCode::OK);
    assert_eq!(iso["method"], "isolation_v1");
    assert!(iso["scores"][0]["score"].as_f64().is_some());

    let (knn_status, knn) = post_json(
        &app,
        "/api/v1/experiments/knn-density",
        serde_json::json!({ "analysis_ids": [a_id, b_id], "k": 1 }),
    )
    .await;
    assert_eq!(knn_status, StatusCode::OK);
    assert_eq!(knn["method"], "knn_density_v1");
    assert_eq!(knn["scores"].as_array().unwrap().len(), 2);
}

async fn test_app_auth_required() -> axum::Router {
    ensure_demo_evolve_repo();
    let mut config = AppConfig::for_tests(workspace_root());
    config.blob_dir = std::env::temp_dir().join(format!("genoma-auth-{}", uuid::Uuid::new_v4()));
    config.auth_required = true;
    let state = AppState::with_backends(config, None, None)
        .await
        .expect("auth test state");
    app(state)
}

async fn post_json_auth(
    app: &axum::Router,
    uri: &str,
    body: Value,
    token: Option<&str>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json");
    if let Some(token) = token {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }
    let response = app
        .clone()
        .oneshot(builder.body(Body::from(serde_json::to_vec(&body).unwrap())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, value)
}

async fn get_auth(app: &axum::Router, uri: &str, token: Option<&str>) -> (StatusCode, Value) {
    let mut builder = Request::builder().method("GET").uri(uri);
    if let Some(token) = token {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }
    let response = app
        .clone()
        .oneshot(builder.body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, value)
}

#[tokio::test]
async fn auth_register_login_me_and_gate_when_required() {
    let app = test_app_auth_required().await;

    let (denied, _) = json_request(&app, "POST", "/api/v1/analyses/demo?file=sample.txt").await;
    assert_eq!(denied, StatusCode::UNAUTHORIZED);

    let email = format!("user-{}@example.com", uuid::Uuid::new_v4());
    let (reg_status, reg) = post_json_auth(
        &app,
        "/api/v1/auth/register",
        serde_json::json!({ "email": email, "password": "password123" }),
        None,
    )
    .await;
    assert_eq!(reg_status, StatusCode::OK);
    let token = reg["token"].as_str().expect("token");
    assert_eq!(reg["user"]["email"], email);

    let (me_status, me) = get_auth(&app, "/api/v1/auth/me", Some(token)).await;
    assert_eq!(me_status, StatusCode::OK);
    assert_eq!(me["email"], email);

    let (login_status, login) = post_json_auth(
        &app,
        "/api/v1/auth/login",
        serde_json::json!({ "email": email, "password": "password123" }),
        None,
    )
    .await;
    assert_eq!(login_status, StatusCode::OK);
    assert!(login["token"].as_str().is_some());

    let (logout_status, _) = post_json_auth(&app, "/api/v1/auth/logout", Value::Null, Some(token)).await;
    assert_eq!(logout_status, StatusCode::OK);
    let (me_after, _) = get_auth(&app, "/api/v1/auth/me", Some(token)).await;
    assert_eq!(me_after, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn auth_demo_with_bearer_when_required() {
    let app = test_app_auth_required().await;
    let email = format!("demo-{}@example.com", uuid::Uuid::new_v4());
    let (reg_status, reg) = post_json_auth(
        &app,
        "/api/v1/auth/register",
        serde_json::json!({ "email": email, "password": "password123" }),
        None,
    )
    .await;
    assert_eq!(reg_status, StatusCode::OK);
    let token = reg["token"].as_str().unwrap();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/analyses/demo?file=sample.txt")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn team_share_round_trip_with_auth_required() {
    let app = test_app_auth_required().await;

    let owner_email = format!("owner-{}@example.com", uuid::Uuid::new_v4());
    let member_email = format!("member-{}@example.com", uuid::Uuid::new_v4());
    let (owner_status, owner) = post_json_auth(
        &app,
        "/api/v1/auth/register",
        serde_json::json!({ "email": owner_email, "password": "password123" }),
        None,
    )
    .await;
    assert_eq!(owner_status, StatusCode::OK);
    let owner_token = owner["token"].as_str().unwrap();

    let (member_status, _) = post_json_auth(
        &app,
        "/api/v1/auth/register",
        serde_json::json!({ "email": member_email, "password": "password123" }),
        None,
    )
    .await;
    assert_eq!(member_status, StatusCode::OK);

    let (team_status, team) = post_json_auth(
        &app,
        "/api/v1/teams",
        serde_json::json!({ "name": "Lab" }),
        Some(owner_token),
    )
    .await;
    assert_eq!(team_status, StatusCode::OK);
    let team_id = team["id"].as_str().unwrap();

    let (invite_status, _) = post_json_auth(
        &app,
        &format!("/api/v1/teams/{team_id}/members"),
        serde_json::json!({ "email": member_email }),
        Some(owner_token),
    )
    .await;
    assert_eq!(invite_status, StatusCode::OK);

    let demo = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/analyses/demo?file=sample.txt")
                .header("authorization", format!("Bearer {owner_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(demo.status(), StatusCode::OK);
    let demo_body = axum::body::to_bytes(demo.into_body(), usize::MAX)
        .await
        .unwrap();
    let demo_json: Value = serde_json::from_slice(&demo_body).unwrap();
    let analysis_id = demo_json["id"].as_str().unwrap();

    let (share_status, _) = post_json_auth(
        &app,
        &format!("/api/v1/analyses/{analysis_id}/share"),
        serde_json::json!({ "team_id": team_id }),
        Some(owner_token),
    )
    .await;
    assert_eq!(share_status, StatusCode::OK);

    let (list_status, listed) = get_auth(
        &app,
        &format!("/api/v1/teams/{team_id}/analyses"),
        Some(owner_token),
    )
    .await;
    assert_eq!(list_status, StatusCode::OK);
    let ids: Vec<&str> = listed
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|item| item["id"].as_str())
        .collect();
    assert!(ids.contains(&analysis_id));
}
