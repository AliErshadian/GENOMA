use std::sync::Arc;

use pi_engine::{CachedPiSource, FilePiSource};

use crate::config::AppConfig;
use crate::progress::ProgressHub;
use crate::storage::FsBlobStore;
use crate::store::MemoryStore;

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub store: MemoryStore,
    pub blobs: FsBlobStore,
    pub progress: ProgressHub,
    pub pi: Arc<CachedPiSource<FilePiSource>>,
}

impl AppState {
    pub async fn new(config: AppConfig) -> crate::error::ApiResult<Self> {
        let blobs = FsBlobStore::new(config.blob_dir.clone()).await?;
        let source = FilePiSource::load(&config.pi_digits_path).map_err(|err| {
            crate::error::ApiError::internal(format!(
                "failed to load π dataset at {}: {err}",
                config.pi_digits_path.display()
            ))
        })?;
        Ok(Self {
            config,
            store: MemoryStore::new(),
            blobs,
            progress: ProgressHub::new(),
            pi: Arc::new(CachedPiSource::new(source, 64)),
        })
    }
}
