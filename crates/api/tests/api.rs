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

async fn test_app() -> axum::Router {
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
