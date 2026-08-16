use analysis_engine::{Anomaly, ProgressEvent, Stage};
use chrono::{DateTime, Utc};
use dna_engine::FileDna;
use serde_json::Value;
use sqlx::postgres::PgPool;
use sqlx::types::Json;
use tracing::warn;
use uuid::Uuid;

use crate::store::AnalysisRecord;

pub async fn save_record(pool: &PgPool, record: &AnalysisRecord) {
    if let Err(err) = save_record_inner(pool, record).await {
        warn!(error = %err, analysis_id = %record.id, "failed to persist analysis");
    }
}

async fn save_record_inner(pool: &PgPool, record: &AnalysisRecord) -> Result<(), sqlx::Error> {
    let status = stage_name(record.status);
    let config = Json(record.config.clone());
    sqlx::query(
        r#"
        INSERT INTO analyses (id, status, original_name, size_bytes, mime_type, storage_key, config, error, created_at, completed_at, owner_user_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        ON CONFLICT (id) DO UPDATE SET
            status = EXCLUDED.status,
            original_name = EXCLUDED.original_name,
            size_bytes = EXCLUDED.size_bytes,
            mime_type = EXCLUDED.mime_type,
            storage_key = EXCLUDED.storage_key,
            config = EXCLUDED.config,
            error = EXCLUDED.error,
            completed_at = EXCLUDED.completed_at,
            owner_user_id = COALESCE(EXCLUDED.owner_user_id, analyses.owner_user_id)
        "#,
    )
    .bind(record.id)
    .bind(&status)
    .bind(&record.original_name)
    .bind(record.size_bytes as i64)
    .bind(&record.mime_type)
    .bind(&record.storage_key)
    .bind(config)
    .bind(&record.error)
    .bind(record.created_at)
    .bind(record.completed_at)
    .bind(record.owner_user_id)
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO file_metadata (id, analysis_id, original_name, size_bytes, mime_type, storage_key)
        VALUES ($1, $1, $2, $3, $4, $5)
        ON CONFLICT (id) DO UPDATE SET
            original_name = EXCLUDED.original_name,
            size_bytes = EXCLUDED.size_bytes,
            mime_type = EXCLUDED.mime_type,
            storage_key = EXCLUDED.storage_key
        "#,
    )
    .bind(record.id)
    .bind(&record.original_name)
    .bind(record.size_bytes as i64)
    .bind(&record.mime_type)
    .bind(&record.storage_key)
    .execute(pool)
    .await?;

    if let Some(progress) = &record.progress {
        sqlx::query(
            r#"
            INSERT INTO analysis_jobs (id, analysis_id, stage, progress, processed_bytes, updated_at)
            VALUES ($1, $1, $2, $3, $4, NOW())
            ON CONFLICT (id) DO UPDATE SET
                stage = EXCLUDED.stage,
                progress = EXCLUDED.progress,
                processed_bytes = EXCLUDED.processed_bytes,
                updated_at = NOW()
            "#,
        )
        .bind(record.id)
        .bind(stage_name(progress.stage))
        .bind(progress.progress)
        .bind(progress.processed_bytes as i64)
        .execute(pool)
        .await?;
    }

    if let Some(dna) = &record.dna {
        sqlx::query("DELETE FROM dna_fingerprints WHERE analysis_id = $1")
            .bind(record.id)
            .execute(pool)
            .await?;
        sqlx::query(
            r#"
            INSERT INTO dna_fingerprints (
                id, analysis_id, chunk_id, raw, pi_derived, visual, pi_offset, pi_wrapped, generator_version, payload
            )
            VALUES ($1, $2, NULL, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(record.id)
        .bind(Json(dna.raw.clone()))
        .bind(Json(dna.pi_derived.clone()))
        .bind(Json(dna.visual.clone()))
        .bind(dna.pi_base_offset as i64)
        .bind(dna.pi_derived.pi_wrapped)
        .bind(&dna.generator_version)
        .bind(Json(dna.clone()))
        .execute(pool)
        .await?;
    }

    sqlx::query("DELETE FROM anomalies WHERE analysis_id = $1")
        .bind(record.id)
        .execute(pool)
        .await?;
    for anomaly in &record.anomalies {
        sqlx::query(
            r#"
            INSERT INTO anomalies (id, analysis_id, chunk_index, offset_bytes, score, details)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(record.id)
        .bind(anomaly.chunk_index as i64)
        .bind(anomaly.offset as i64)
        .bind(anomaly.score)
        .bind(Json(anomaly.clone()))
        .execute(pool)
        .await?;
    }

    Ok(())
}

pub async fn load_record(pool: &PgPool, id: Uuid) -> Option<AnalysisRecord> {
    load_record_inner(pool, id)
        .await
        .map_err(|err| {
            warn!(error = %err, %id, "failed to load analysis");
            err
        })
        .ok()
        .flatten()
}

async fn load_record_inner(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<AnalysisRecord>, sqlx::Error> {
    let row = sqlx::query_as::<_, AnalysisRow>(
        r#"
        SELECT id, status, original_name, size_bytes, mime_type, storage_key, config, error, created_at, completed_at, owner_user_id
        FROM analyses
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let job = sqlx::query_as::<_, JobRow>(
        r#"
        SELECT stage, progress, processed_bytes
        FROM analysis_jobs
        WHERE analysis_id = $1
        ORDER BY updated_at DESC
        LIMIT 1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    let payload: Option<Json<FileDna>> = sqlx::query_scalar(
        "SELECT payload FROM dna_fingerprints WHERE analysis_id = $1 AND chunk_id IS NULL LIMIT 1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    let details: Vec<Json<Anomaly>> = sqlx::query_scalar(
        "SELECT details FROM anomalies WHERE analysis_id = $1 ORDER BY score DESC",
    )
    .bind(id)
    .fetch_all(pool)
    .await?;

    let status = parse_stage(&row.status);
    let progress = job.map(|job| ProgressEvent {
        stage: parse_stage(&job.stage),
        progress: job.progress,
        processed_bytes: job.processed_bytes as u64,
        total_bytes: Some(row.size_bytes as u64),
        message: row
            .error
            .clone()
            .unwrap_or_else(|| job_message(parse_stage(&job.stage))),
    });

    Ok(Some(AnalysisRecord {
        id: row.id,
        status,
        original_name: row.original_name,
        size_bytes: row.size_bytes as u64,
        mime_type: row.mime_type,
        storage_key: row.storage_key,
        config: row.config.0,
        error: row.error,
        created_at: row.created_at,
        completed_at: row.completed_at,
        dna: payload.map(|json| json.0),
        anomalies: details.into_iter().map(|json| json.0).collect(),
        progress,
        owner_user_id: row.owner_user_id,
    }))
}

pub async fn list_records(pool: &PgPool, limit: i64) -> Vec<Uuid> {
    sqlx::query_scalar(
        "SELECT id FROM analyses ORDER BY created_at DESC LIMIT $1",
    )
    .bind(limit)
    .fetch_all(pool)
    .await
    .unwrap_or_else(|err| {
        warn!(error = %err, "failed to list analyses");
        Vec::new()
    })
}

#[derive(sqlx::FromRow)]
struct AnalysisRow {
    id: Uuid,
    status: String,
    original_name: String,
    size_bytes: i64,
    mime_type: Option<String>,
    storage_key: String,
    config: Json<genoma_core::AnalysisConfig>,
    error: Option<String>,
    created_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
    owner_user_id: Option<Uuid>,
}

#[derive(sqlx::FromRow)]
struct JobRow {
    stage: String,
    progress: f64,
    processed_bytes: i64,
}

fn stage_name(stage: Stage) -> String {
    serde_json::to_value(stage)
        .ok()
        .and_then(|value| match value {
            Value::String(name) => Some(name),
            _ => None,
        })
        .unwrap_or_else(|| "QUEUED".to_string())
}

fn parse_stage(name: &str) -> Stage {
    serde_json::from_value(Value::String(name.to_string())).unwrap_or(Stage::Queued)
}

fn job_message(stage: Stage) -> String {
    match stage {
        Stage::Queued => "Queued".into(),
        Stage::ReadingFile => "Reading file".into(),
        Stage::ExtractingFeatures => "Extracting features".into(),
        Stage::GeneratingPidna => "Generating πDNA".into(),
        Stage::DetectingAnomalies => "Detecting anomalies".into(),
        Stage::BuildingVisualization => "Building visualization".into(),
        Stage::Complete => "GENOMA complete".into(),
        Stage::Failed => "Analysis failed".into(),
    }
}
