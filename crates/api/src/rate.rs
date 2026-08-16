use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::IntoResponse;

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Clone)]
pub struct RateGate {
    max: u64,
    window: Duration,
    hits: Arc<Mutex<VecDeque<Instant>>>,
}

impl RateGate {
    pub fn new(max_per_minute: u64) -> Self {
        Self {
            max: max_per_minute.max(1),
            window: Duration::from_secs(60),
            hits: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn allow(&self) -> bool {
        let now = Instant::now();
        let mut hits = self.hits.lock().unwrap_or_else(|err| err.into_inner());
        while hits.front().is_some_and(|time| now.duration_since(*time) > self.window) {
            hits.pop_front();
        }
        if hits.len() as u64 >= self.max {
            return false;
        }
        hits.push_back(now);
        true
    }
}

pub async fn rate_middleware(State(state): State<AppState>, request: Request, next: Next) -> impl IntoResponse {
    if !state.rate.allow() {
        return ApiError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "rate_limited",
            "too many requests",
        )
        .into_response();
    }
    next.run(request).await
}
