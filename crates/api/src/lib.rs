pub mod config;
pub mod db;
pub mod error;
pub mod evolution;
pub mod git_import;
pub mod jobs;
pub mod persist;
pub mod progress;
pub mod rate;
pub mod routes;
pub mod security;
pub mod state;
pub mod storage;
pub mod store;

use std::time::Duration;

use axum::extract::DefaultBodyLimit;
use axum::middleware;
use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use crate::config::AppConfig;
use crate::rate::rate_middleware;
use crate::routes::{
    compare_analyses, create_analysis, create_demo, create_evolution, create_evolution_from_git,
    detect_mutations, experiment_isolation, experiment_knn_density, galaxy, get_analysis,
    get_anomalies, get_dna, get_evolution, health, list_analyses, list_demos, list_evolution,
    not_implemented, progress_latest, progress_sse,
};
use crate::state::AppState;

pub async fn build_state(config: AppConfig) -> crate::error::ApiResult<AppState> {
    AppState::new(config).await
}

pub fn app(state: AppState) -> Router {
    let max = state.config.max_upload_bytes.min(usize::MAX as u64) as usize;
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/api/v1/health", get(health))
        .route("/api/v1/analyses", get(list_analyses).post(create_analysis))
        .route("/api/v1/analyses/demo", post(create_demo))
        .route("/api/v1/analyses/{id}", get(get_analysis))
        .route(
            "/api/v1/analyses/{id}/progress/latest",
            get(progress_latest),
        )
        .route("/api/v1/analyses/{id}/progress", get(progress_sse))
        .route("/api/v1/demos", get(list_demos))
        .route("/api/v1/dna/{id}", get(get_dna))
        .route("/api/v1/anomalies/{id}", get(get_anomalies))
        .route("/api/v1/compare", post(compare_analyses))
        .route("/api/v1/mutations", post(detect_mutations))
        .route("/api/v1/galaxy", post(galaxy))
        .route("/api/v1/evolution", get(list_evolution).post(create_evolution))
        .route("/api/v1/evolution/git", post(create_evolution_from_git))
        .route("/api/v1/evolution/{id}", get(get_evolution))
        .route("/api/v1/experiments/isolation", post(experiment_isolation))
        .route(
            "/api/v1/experiments/knn-density",
            post(experiment_knn_density),
        )
        .route("/api/v1/export", post(not_implemented))
        .layer(DefaultBodyLimit::max(max))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            rate_middleware,
        ))
        .layer(TimeoutLayer::with_status_code(
            axum::http::StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(60 * 30),
        ))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}
