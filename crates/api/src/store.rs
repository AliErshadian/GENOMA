use std::collections::HashMap;
use std::sync::Arc;

use analysis_engine::{Anomaly, ProgressEvent, Stage};
use chrono::{DateTime, Utc};
use dna_engine::FileDna;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::persist;

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
    pub owner_user_id: Option<Uuid>,
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

    pub async fn update<F>(&self, id: Uuid, mutator: F) -> Option<AnalysisRecord>
    where
        F: FnOnce(&mut AnalysisRecord),
    {
        let mut guard = self.inner.write().await;
        if let Some(record) = guard.get_mut(&id) {
            mutator(record);
            return Some(record.clone());
        }
        None
    }

    pub async fn list(&self) -> Vec<AnalysisRecord> {
        self.inner.read().await.values().cloned().collect()
    }
}

#[derive(Clone)]
pub struct AnalysisStore {
    memory: MemoryStore,
    postgres: Option<PgPool>,
}

impl AnalysisStore {
    pub fn new(postgres: Option<PgPool>) -> Self {
        Self {
            memory: MemoryStore::new(),
            postgres,
        }
    }

    pub async fn insert(&self, record: AnalysisRecord) {
        if let Some(pool) = &self.postgres {
            persist::save_record(pool, &record).await;
        }
        self.memory.insert(record).await;
    }

    pub async fn get(&self, id: Uuid) -> Option<AnalysisRecord> {
        if let Some(record) = self.memory.get(id).await {
            return Some(record);
        }
        let pool = self.postgres.as_ref()?;
        let record = persist::load_record(pool, id).await?;
        self.memory.insert(record.clone()).await;
        Some(record)
    }

    pub async fn update<F>(&self, id: Uuid, mutator: F) -> Option<AnalysisRecord>
    where
        F: FnOnce(&mut AnalysisRecord),
    {
        let updated = self.memory.update(id, mutator).await;
        if let (Some(pool), Some(record)) = (&self.postgres, &updated) {
            persist::save_record(pool, record).await;
        }
        updated
    }

    pub async fn list(&self) -> Vec<AnalysisRecord> {
        let mut by_id: HashMap<Uuid, AnalysisRecord> = self
            .memory
            .list()
            .await
            .into_iter()
            .map(|record| (record.id, record))
            .collect();
        if let Some(pool) = &self.postgres {
            for id in persist::list_records(pool, 50).await {
                if let std::collections::hash_map::Entry::Vacant(entry) = by_id.entry(id) {
                    if let Some(record) = persist::load_record(pool, id).await {
                        entry.insert(record);
                    }
                }
            }
        }
        let mut records: Vec<AnalysisRecord> = by_id.into_values().collect();
        records.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        records.truncate(50);
        records
    }
}
