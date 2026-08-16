use std::collections::HashMap;
use std::sync::Arc;

use analysis_engine::{Anomaly, ProgressEvent, Stage};
use chrono::{DateTime, Utc};
use dna_engine::FileDna;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnalysisRecord {
    pub id: Uuid,
    pub status: Stage,
    pub original_name: String,
    pub size_bytes: u64,
    pub mime_type: Option<String>,
    pub storage_key: String,
    pub config: genoma_core::AnalysisConfig,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub dna: Option<FileDna>,
    pub anomalies: Vec<Anomaly>,
    pub progress: Option<ProgressEvent>,
}

#[derive(Clone, Default)]
pub struct MemoryStore {
    inner: Arc<RwLock<HashMap<Uuid, AnalysisRecord>>>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn insert(&self, record: AnalysisRecord) {
        self.inner.write().await.insert(record.id, record);
    }

    pub async fn get(&self, id: Uuid) -> Option<AnalysisRecord> {
        self.inner.read().await.get(&id).cloned()
    }

    pub async fn update<F>(&self, id: Uuid, mutator: F)
    where
        F: FnOnce(&mut AnalysisRecord),
    {
        if let Some(record) = self.inner.write().await.get_mut(&id) {
            mutator(record);
        }
    }

    pub async fn list(&self) -> Vec<AnalysisRecord> {
        self.inner.read().await.values().cloned().collect()
    }
}
