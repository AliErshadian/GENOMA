use std::sync::Arc;

use analysis_engine::ProgressEvent;
use pi_engine::{CachedPiSource, FilePiSource};
use redis::AsyncCommands;
use tracing::warn;
use uuid::Uuid;

use crate::auth::AuthStore;
use crate::config::AppConfig;
use crate::evolution::EvolutionStore;
use crate::progress::ProgressHub;
use crate::storage::BlobStore;
use crate::store::AnalysisStore;
use crate::teams::TeamStore;

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub store: AnalysisStore,
    pub evolution: EvolutionStore,
    pub auth: AuthStore,
    pub teams: TeamStore,
    pub blobs: BlobStore,
    pub progress: ProgressHub,
    pub redis: Option<redis::aio::MultiplexedConnection>,
    pub pi: Arc<CachedPiSource<FilePiSource>>,
    pub rate: crate::rate::RateGate,
}

impl AppState {
    pub async fn new(config: AppConfig) -> crate::error::ApiResult<Self> {
        let postgres = crate::db::connect_postgres(&config).await;
        let redis = crate::db::connect_redis(&config).await;
        Self::with_backends(config, postgres, redis).await
    }

    pub async fn with_backends(
        config: AppConfig,
        postgres: Option<sqlx::PgPool>,
        redis: Option<redis::aio::MultiplexedConnection>,
    ) -> crate::error::ApiResult<Self> {
        let blobs = BlobStore::from_config(&config).await?;
        let source = FilePiSource::load(&config.pi_digits_path).map_err(|err| {
            crate::error::ApiError::internal(format!(
                "failed to load π dataset at {}: {err}",
                config.pi_digits_path.display()
            ))
        })?;
        let rate = crate::rate::RateGate::new(config.rate_limit_per_minute);
        Ok(Self {
            config,
            store: AnalysisStore::new(postgres.clone()),
            evolution: EvolutionStore::new(postgres.clone()),
            auth: AuthStore::new(postgres.clone()),
            teams: TeamStore::new(postgres),
            blobs,
            progress: ProgressHub::new(),
            redis,
            pi: Arc::new(CachedPiSource::new(source, 64)),
            rate,
        })
    }

    pub async fn publish_progress(&self, id: Uuid, event: ProgressEvent) {
        self.progress.publish(id, event.clone());
        if let Some(connection) = &self.redis {
            let mut connection = connection.clone();
            let key = format!("analysis:{id}:progress");
            if let Ok(payload) = serde_json::to_string(&event) {
                let result: Result<(), _> = connection.set_ex(key, payload, 3600).await;
                if let Err(err) = result {
                    warn!(error = %err, %id, "failed to cache progress in Redis");
                }
            }
        }
    }

    pub async fn latest_progress(&self, id: Uuid) -> Option<ProgressEvent> {
        if let Some(connection) = &self.redis {
            let mut connection = connection.clone();
            let key = format!("analysis:{id}:progress");
            if let Ok(payload) = connection.get::<_, Option<String>>(key).await {
                if let Some(payload) = payload {
                    if let Ok(event) = serde_json::from_str(&payload) {
                        return Some(event);
                    }
                }
            }
        }
        self.store.get(id).await.and_then(|record| record.progress)
    }
}
