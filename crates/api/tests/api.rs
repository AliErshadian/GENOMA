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
