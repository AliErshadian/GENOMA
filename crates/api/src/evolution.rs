use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tokio::sync::RwLock;
use tracing::warn;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EvolutionSnapshotRecord {
    pub id: Uuid,
    pub series_id: Uuid,
    pub analysis_id: Uuid,
    pub version_label: String,
    pub file_name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EvolutionSeriesRecord {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub snapshots: Vec<EvolutionSnapshotRecord>,
}

#[derive(Clone, Default)]
struct MemoryEvolution {
    series: HashMap<Uuid, EvolutionSeriesRecord>,
}

#[derive(Clone)]
pub struct EvolutionStore {
    memory: Arc<RwLock<MemoryEvolution>>,
    postgres: Option<PgPool>,
}

impl EvolutionStore {
    pub fn new(postgres: Option<PgPool>) -> Self {
        Self {
            memory: Arc::new(RwLock::new(MemoryEvolution::default())),
            postgres,
        }
    }

    pub async fn insert(&self, series: EvolutionSeriesRecord) {
        if let Some(pool) = &self.postgres {
            if let Err(err) = persist_series(pool, &series).await {
                warn!(error = %err, series_id = %series.id, "failed to persist evolution series");
            }
        }
        self.memory.write().await.series.insert(series.id, series);
    }

    pub async fn get(&self, id: Uuid) -> Option<EvolutionSeriesRecord> {
        if let Some(series) = self.memory.read().await.series.get(&id).cloned() {
            return Some(series);
        }
        let pool = self.postgres.as_ref()?;
        let series = load_series(pool, id).await?;
        self.memory
            .write()
            .await
            .series
            .insert(series.id, series.clone());
        Some(series)
    }

    pub async fn list(&self, limit: usize) -> Vec<EvolutionSeriesRecord> {
        let mut by_id: HashMap<Uuid, EvolutionSeriesRecord> = self
            .memory
            .read()
            .await
            .series
            .values()
            .cloned()
            .map(|series| (series.id, series))
            .collect();
        if let Some(pool) = &self.postgres {
            for id in list_series_ids(pool, limit as i64).await {
                if let std::collections::hash_map::Entry::Vacant(entry) = by_id.entry(id) {
                    if let Some(series) = load_series(pool, id).await {
                        entry.insert(series);
                    }
                }
            }
        }
        let mut series: Vec<EvolutionSeriesRecord> = by_id.into_values().collect();
        series.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        series.truncate(limit);
        series
    }
}

async fn persist_series(pool: &PgPool, series: &EvolutionSeriesRecord) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO evolution_series (id, name, created_at)
        VALUES ($1, $2, $3)
        ON CONFLICT (id) DO UPDATE SET
            name = EXCLUDED.name,
            created_at = EXCLUDED.created_at
        "#,
    )
    .bind(series.id)
    .bind(&series.name)
    .bind(series.created_at)
    .execute(pool)
    .await?;

    for snapshot in &series.snapshots {
        sqlx::query(
            r#"
            INSERT INTO evolution_snapshots (id, series_id, analysis_id, version_label, created_at)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (id) DO UPDATE SET
                series_id = EXCLUDED.series_id,
                analysis_id = EXCLUDED.analysis_id,
                version_label = EXCLUDED.version_label,
                created_at = EXCLUDED.created_at
            "#,
        )
        .bind(snapshot.id)
        .bind(snapshot.series_id)
        .bind(snapshot.analysis_id)
        .bind(&snapshot.version_label)
        .bind(snapshot.created_at)
        .execute(pool)
        .await?;
    }
    Ok(())
}

async fn list_series_ids(pool: &PgPool, limit: i64) -> Vec<Uuid> {
    sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT id
        FROM evolution_series
        ORDER BY created_at DESC
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
}

async fn load_series(pool: &PgPool, series_id: Uuid) -> Option<EvolutionSeriesRecord> {
    #[derive(sqlx::FromRow)]
    struct SeriesRow {
        id: Uuid,
        name: String,
        created_at: DateTime<Utc>,
    }

    let series = sqlx::query_as::<_, SeriesRow>(
        r#"
        SELECT id, name, created_at
        FROM evolution_series
        WHERE id = $1
        "#,
    )
    .bind(series_id)
    .fetch_optional(pool)
    .await
    .ok()??;

    #[derive(sqlx::FromRow)]
    struct SnapshotRow {
        id: Uuid,
        series_id: Uuid,
        analysis_id: Uuid,
        version_label: String,
        created_at: DateTime<Utc>,
        original_name: Option<String>,
    }

    let rows = sqlx::query_as::<_, SnapshotRow>(
        r#"
        SELECT e.id, e.series_id, e.analysis_id, e.version_label, e.created_at, a.original_name
        FROM evolution_snapshots e
        LEFT JOIN analyses a ON a.id = e.analysis_id
        WHERE e.series_id = $1
        ORDER BY e.created_at ASC
        "#,
    )
    .bind(series_id)
    .fetch_all(pool)
    .await
    .ok()?;

    let snapshots = rows
        .into_iter()
        .map(|row| EvolutionSnapshotRecord {
            id: row.id,
            series_id: row.series_id,
            analysis_id: row.analysis_id,
            version_label: row.version_label,
            file_name: row.original_name.unwrap_or_else(|| "unknown".into()),
            created_at: row.created_at,
        })
        .collect();

    Some(EvolutionSeriesRecord {
        id: series.id,
        name: series.name,
        created_at: series.created_at,
        snapshots,
    })
}
