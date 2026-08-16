use std::convert::Infallible;
use std::time::Duration;

use analysis_engine::Stage;
use axum::extract::{Multipart, Path, Query, State};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::Json;
use chrono::Utc;
use futures_util::stream;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{ApiError, ApiResult};
use crate::jobs::spawn_analysis_job;
use crate::security::{sniff_mime, validate_filename, validate_size};
use crate::state::AppState;
use crate::storage::object_key;
use crate::store::AnalysisRecord;

#[derive(Serialize)]
pub struct HealthResponse {
    pub name: &'static str,
    pub version: &'static str,
    pub generator: &'static str,
}

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        name: genoma_core::PRODUCT_NAME,
        version: env!("CARGO_PKG_VERSION"),
        generator: genoma_core::GENERATOR_VERSION,
    })
}

#[derive(Deserialize)]
pub struct AnalysisQuery {
    pub chunk_size: Option<usize>,
    pub level: Option<String>,
    pub pi_offset: Option<u64>,
}

#[derive(Serialize)]
pub struct AnalysisSummary {
    pub id: Uuid,
    pub status: Stage,
    pub original_name: String,
    pub size_bytes: u64,
    pub mime_type: Option<String>,
    pub created_at: chrono::DateTime<Utc>,
    pub completed_at: Option<chrono::DateTime<Utc>>,
    pub progress: Option<analysis_engine::ProgressEvent>,
    pub dna: Option<FileSummary>,
    pub anomalies: usize,
}

#[derive(Serialize)]
pub struct FileSummary {
    pub entropy: f64,
    pub complexity: f64,
    pub repetition: f64,
    pub anomalies: usize,
    pub mutations: usize,
    pub pi_offset: u64,
    pub chunk_count: u64,
    pub generator_version: String,
}

impl From<AnalysisRecord> for AnalysisSummary {
    fn from(record: AnalysisRecord) -> Self {
        let anomalies = record.anomalies.len();
        let dna = record.dna.as_ref().map(|dna| FileSummary {
            entropy: dna.raw.entropy,
            complexity: dna.raw.complexity,
            repetition: dna.raw.repetition,
            anomalies,
            mutations: 0,
            pi_offset: dna.pi_base_offset,
            chunk_count: dna.chunk_count,
            generator_version: dna.generator_version.clone(),
        });
        Self {
            id: record.id,
            status: record.status,
            original_name: record.original_name,
            size_bytes: record.size_bytes,
            mime_type: record.mime_type,
            created_at: record.created_at,
            completed_at: record.completed_at,
            progress: record.progress,
            dna,
            anomalies,
        }
    }
}

pub async fn create_analysis(
    State(state): State<AppState>,
    Query(query): Query<AnalysisQuery>,
    mut multipart: Multipart,
) -> ApiResult<Json<AnalysisSummary>> {
    let mut filename = None;
    let mut mime = None;
    let mut stored_key = None;
    let mut size = 0_u64;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|err| ApiError::bad_request(err.to_string()))?
    {
        let name = field.name().unwrap_or_default().to_string();
        if name != "file" {
            continue;
        }
        let given = field
            .file_name()
            .map(ToOwned::to_owned)
            .ok_or_else(|| ApiError::bad_request("missing file name"))?;
        validate_filename(&given).map_err(ApiError::bad_request)?;
        mime = Some(sniff_mime(&given, field.content_type()));
        filename = Some(given.clone());
        let id = Uuid::new_v4();
        let key = object_key(id, &given);
        size = state
            .blobs
            .put_field(&key, field, state.config.max_upload_bytes)
            .await?;
        if size == 0 {
            return Err(ApiError::bad_request("empty file"));
        }
        stored_key = Some((id, key));
        break;
    }

    let filename = filename.ok_or_else(|| ApiError::bad_request("file field is required"))?;
    let (id, storage_key) = stored_key.unwrap();
    let mut config = state.config.default_analysis.clone();
    if let Some(size) = query.chunk_size {
        config.chunk_size = genoma_core::ChunkSize::from_bytes(size)
            .map_err(|err| ApiError::bad_request(err.to_string()))?;
    }
    if let Some(level) = query.level {
        config.level = level
            .parse()
            .map_err(|err: genoma_core::Error| ApiError::bad_request(err.to_string()))?;
    }
    if let Some(offset) = query.pi_offset {
        config.pi_base_offset = offset;
    }

    let record = AnalysisRecord {
        id,
        status: Stage::Queued,
        original_name: filename,
        size_bytes: size,
        mime_type: mime,
        storage_key,
        config,
        error: None,
        created_at: Utc::now(),
        completed_at: None,
        dna: None,
        anomalies: Vec::new(),
        progress: Some(analysis_engine::ProgressEvent {
            stage: Stage::Queued,
            progress: 0.0,
            processed_bytes: 0,
            total_bytes: Some(size),
            message: "Queued".to_string(),
        }),
    };
    state.store.insert(record.clone()).await;
    spawn_analysis_job(state.clone(), id);
    Ok(Json(record.into()))
}

pub async fn list_analyses(State(state): State<AppState>) -> Json<Vec<AnalysisSummary>> {
    let records = state.store.list().await;
    Json(records.into_iter().map(AnalysisSummary::from).collect())
}

pub async fn get_analysis(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<AnalysisSummary>> {
    let record = state
        .store
        .get(id)
        .await
        .ok_or_else(|| ApiError::not_found("analysis not found"))?;
    Ok(Json(record.into()))
}

pub async fn get_dna(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<dna_engine::FileDna>> {
    let record = state
        .store
        .get(id)
        .await
        .ok_or_else(|| ApiError::not_found("analysis not found"))?;
    record
        .dna
        .ok_or_else(|| {
            ApiError::new(
                axum::http::StatusCode::ACCEPTED,
                "pending",
                "analysis is not complete",
            )
        })
        .map(Json)
}

pub async fn get_anomalies(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Vec<analysis_engine::Anomaly>>> {
    let record = state
        .store
        .get(id)
        .await
        .ok_or_else(|| ApiError::not_found("analysis not found"))?;
    Ok(Json(record.anomalies))
}

#[derive(Deserialize)]
pub struct CompareRequest {
    pub left_id: Uuid,
    pub right_id: Uuid,
}

#[derive(Serialize)]
pub struct CompareResponse {
    pub left_id: Uuid,
    pub right_id: Uuid,
    pub left_name: String,
    pub right_name: String,
    pub similarity: analysis_engine::SimilarityBreakdown,
}

async fn load_completed_dna(
    state: &AppState,
    id: Uuid,
    label: &str,
) -> ApiResult<(String, dna_engine::FileDna)> {
    let record = state
        .store
        .get(id)
        .await
        .ok_or_else(|| ApiError::not_found(format!("{label} analysis not found")))?;
    if record.status != Stage::Complete {
        return Err(ApiError::conflict(format!("{label} analysis is not complete")));
    }
    let dna = record
        .dna
        .ok_or_else(|| ApiError::conflict(format!("{label} analysis is not complete")))?;
    Ok((record.original_name, dna))
}

pub async fn compare_analyses(
    State(state): State<AppState>,
    Json(body): Json<CompareRequest>,
) -> ApiResult<Json<CompareResponse>> {
    let (left_name, left_dna) = load_completed_dna(&state, body.left_id, "left").await?;
    let (right_name, right_dna) = load_completed_dna(&state, body.right_id, "right").await?;
    let similarity = analysis_engine::compare_dna(
        &left_dna,
        &right_dna,
        analysis_engine::SimilarityWeights::default(),
    );
    Ok(Json(CompareResponse {
        left_id: body.left_id,
        right_id: body.right_id,
        left_name,
        right_name,
        similarity,
    }))
}

#[derive(Deserialize)]
pub struct MutationsRequest {
    pub baseline_id: Uuid,
    pub current_id: Uuid,
}

#[derive(Serialize)]
pub struct MutationsResponse {
    pub baseline_id: Uuid,
    pub current_id: Uuid,
    pub baseline_name: String,
    pub current_name: String,
    pub mutations: Vec<analysis_engine::Mutation>,
}

pub async fn detect_mutations(
    State(state): State<AppState>,
    Json(body): Json<MutationsRequest>,
) -> ApiResult<Json<MutationsResponse>> {
    let (baseline_name, baseline_dna) =
        load_completed_dna(&state, body.baseline_id, "baseline").await?;
    let (current_name, current_dna) =
        load_completed_dna(&state, body.current_id, "current").await?;
    let mutations = analysis_engine::detect_mutations(&baseline_dna, &current_dna);
    Ok(Json(MutationsResponse {
        baseline_id: body.baseline_id,
        current_id: body.current_id,
        baseline_name,
        current_name,
        mutations,
    }))
}

#[derive(Deserialize)]
pub struct GalaxyRequest {
    pub analysis_ids: Vec<Uuid>,
}

#[derive(Serialize)]
pub struct GalaxyNode {
    pub id: Uuid,
    pub name: String,
    pub size_bytes: u64,
    pub entropy: f64,
    pub complexity: f64,
    pub repetition: f64,
    pub chunk_count: u64,
    pub generator_version: String,
    pub cluster_id: u32,
    pub position: [f64; 3],
}

#[derive(Serialize)]
pub struct GalaxyEdge {
    pub from: Uuid,
    pub to: Uuid,
    pub strength: f64,
}

#[derive(Serialize)]
pub struct GalaxyResponse {
    pub nodes: Vec<GalaxyNode>,
    pub cluster_count: u32,
    pub links: Vec<GalaxyEdge>,
}

pub async fn galaxy(
    State(state): State<AppState>,
    Json(body): Json<GalaxyRequest>,
) -> ApiResult<Json<GalaxyResponse>> {
    let mut seen = std::collections::HashSet::new();
    let mut ids = Vec::new();
    for id in body.analysis_ids {
        if seen.insert(id) {
            ids.push(id);
        }
    }
    if ids.is_empty() {
        return Err(ApiError::bad_request("analysis_ids must not be empty"));
    }
    if ids.len() > 50 {
        return Err(ApiError::bad_request("analysis_ids capped at 50"));
    }

    let mut records = Vec::with_capacity(ids.len());
    let mut dnas = Vec::with_capacity(ids.len());
    for id in &ids {
        let record = state
            .store
            .get(*id)
            .await
            .ok_or_else(|| ApiError::not_found(format!("analysis not found: {id}")))?;
        if record.status != Stage::Complete {
            return Err(ApiError::conflict(format!("analysis is not complete: {id}")));
        }
        let dna = record
            .dna
            .clone()
            .ok_or_else(|| ApiError::conflict(format!("analysis is not complete: {id}")))?;
        dnas.push(dna);
        records.push(record);
    }

    let labels = analysis_engine::cluster_files(&dnas);
    let cluster_count = labels.iter().copied().max().map(|m| m + 1).unwrap_or(0);
    let (positions, embed_links) = analysis_engine::embed_files(&dnas);

    let mut nodes = Vec::with_capacity(ids.len());
    for (idx, id) in ids.iter().copied().enumerate() {
        let record = &records[idx];
        let dna = &dnas[idx];
        nodes.push(GalaxyNode {
            id,
            name: record.original_name.clone(),
            size_bytes: record.size_bytes,
            entropy: dna.raw.entropy,
            complexity: dna.raw.complexity,
            repetition: dna.raw.repetition,
            chunk_count: dna.chunk_count,
            generator_version: dna.generator_version.clone(),
            cluster_id: labels[idx],
            position: positions[idx],
        });
    }

    let links = embed_links
        .into_iter()
        .map(|link| GalaxyEdge {
            from: ids[link.from],
            to: ids[link.to],
            strength: link.strength,
        })
        .collect();

    Ok(Json(GalaxyResponse {
        nodes,
        cluster_count,
        links,
    }))
}

pub async fn progress_latest(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<analysis_engine::ProgressEvent>> {
    let event = state
        .latest_progress(id)
        .await
        .ok_or_else(|| ApiError::not_found("analysis not found"))?;
    Ok(Json(event))
}

pub async fn progress_sse(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>>> {
    let record = state
        .store
        .get(id)
        .await
        .ok_or_else(|| ApiError::not_found("analysis not found"))?;
    let rx = state.progress.subscribe();
    let initial = state
        .latest_progress(id)
        .await
        .or(record.progress.clone());
    let stream = stream::once(async move {
        if let Some(event) = initial {
            let json = serde_json::to_string(&event).unwrap_or_else(|_| "{}".into());
            Ok(Event::default().data(json))
        } else {
            Ok(Event::default().comment("waiting"))
        }
    })
    .chain(async_stream(rx, id));

    Ok(Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15))))
}

fn async_stream(
    rx: tokio::sync::broadcast::Receiver<(Uuid, analysis_engine::ProgressEvent)>,
    id: Uuid,
) -> impl tokio_stream::Stream<Item = Result<Event, Infallible>> {
    futures_util::stream::unfold(rx, move |mut rx| async move {
        loop {
            match rx.recv().await {
                Ok((event_id, event)) if event_id == id => {
                    let json = serde_json::to_string(&event).unwrap_or_else(|_| "{}".into());
                    let item = Ok(Event::default().data(json));
                    return Some((item, rx));
                }
                Ok(_) => continue,
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(_) => return None,
            }
        }
    })
}

#[derive(Deserialize)]
pub struct DemoQuery {
    pub file: Option<String>,
    pub chunk_size: Option<usize>,
    pub level: Option<String>,
    pub pi_offset: Option<u64>,
}

pub async fn create_demo(
    State(state): State<AppState>,
    Query(query): Query<DemoQuery>,
) -> ApiResult<Json<AnalysisSummary>> {
    let name = query.file.unwrap_or_else(|| "sample.txt".to_string());
    validate_filename(&name).map_err(ApiError::bad_request)?;
    let path = state.config.demo_dir.join(&name);
    let bytes = tokio::fs::read(&path)
        .await
        .map_err(|_| ApiError::not_found(format!("demo file not found: {name}")))?;
    validate_size(bytes.len() as u64, state.config.max_upload_bytes)
        .map_err(ApiError::payload_too_large)?;
    let id = Uuid::new_v4();
    let storage_key = object_key(id, &name);
    state.blobs.put_bytes(&storage_key, &bytes).await?;
    let mut config = state.config.default_analysis.clone();
    if let Some(size) = query.chunk_size {
        config.chunk_size = genoma_core::ChunkSize::from_bytes(size)
            .map_err(|err| ApiError::bad_request(err.to_string()))?;
    }
    if let Some(level) = query.level {
        config.level = level
            .parse()
            .map_err(|err: genoma_core::Error| ApiError::bad_request(err.to_string()))?;
    }
    if let Some(offset) = query.pi_offset {
        config.pi_base_offset = offset;
    }
    let record = AnalysisRecord {
        id,
        status: Stage::Queued,
        original_name: name.clone(),
        size_bytes: bytes.len() as u64,
        mime_type: Some(sniff_mime(&name, None)),
        storage_key,
        config,
        error: None,
        created_at: Utc::now(),
        completed_at: None,
        dna: None,
        anomalies: Vec::new(),
        progress: Some(analysis_engine::ProgressEvent {
            stage: Stage::Queued,
            progress: 0.0,
            processed_bytes: 0,
            total_bytes: Some(bytes.len() as u64),
            message: "Queued demo analysis".to_string(),
        }),
    };
    state.store.insert(record.clone()).await;
    spawn_analysis_job(state, id);
    Ok(Json(record.into()))
}

pub async fn list_demos(State(state): State<AppState>) -> ApiResult<Json<Vec<String>>> {
    let mut names = Vec::new();
    let mut dir = tokio::fs::read_dir(&state.config.demo_dir)
        .await
        .map_err(|err| ApiError::internal(err.to_string()))?;
    while let Some(entry) = dir
        .next_entry()
        .await
        .map_err(|err| ApiError::internal(err.to_string()))?
    {
        if entry
            .file_type()
            .await
            .map(|kind| kind.is_file())
            .unwrap_or(false)
        {
            if let Some(name) = entry.file_name().to_str() {
                names.push(name.to_string());
            }
        }
    }
    names.sort();
    Ok(Json(names))
}

#[derive(Deserialize)]
pub struct EvolutionSnapshotInput {
    pub analysis_id: Uuid,
    pub version_label: String,
}

#[derive(Deserialize)]
pub struct CreateEvolutionRequest {
    pub name: Option<String>,
    pub snapshots: Vec<EvolutionSnapshotInput>,
}

#[derive(Serialize)]
pub struct EvolutionSnapshotResponse {
    pub id: Uuid,
    pub analysis_id: Uuid,
    pub version_label: String,
    pub file_name: String,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Serialize)]
pub struct EvolutionSeriesResponse {
    pub id: Uuid,
    pub name: String,
    pub created_at: chrono::DateTime<Utc>,
    pub snapshots: Vec<EvolutionSnapshotResponse>,
}

impl From<crate::evolution::EvolutionSeriesRecord> for EvolutionSeriesResponse {
    fn from(series: crate::evolution::EvolutionSeriesRecord) -> Self {
        Self {
            id: series.id,
            name: series.name,
            created_at: series.created_at,
            snapshots: series
                .snapshots
                .into_iter()
                .map(|snapshot| EvolutionSnapshotResponse {
                    id: snapshot.id,
                    analysis_id: snapshot.analysis_id,
                    version_label: snapshot.version_label,
                    file_name: snapshot.file_name,
                    created_at: snapshot.created_at,
                })
                .collect(),
        }
    }
}

pub async fn create_evolution(
    State(state): State<AppState>,
    Json(body): Json<CreateEvolutionRequest>,
) -> ApiResult<Json<EvolutionSeriesResponse>> {
    if body.snapshots.is_empty() {
        return Err(ApiError::bad_request("snapshots must not be empty"));
    }
    if body.snapshots.len() > 20 {
        return Err(ApiError::bad_request("snapshots capped at 20"));
    }

    let series_id = Uuid::new_v4();
    let created_at = Utc::now();
    let mut seen = std::collections::HashSet::new();
    let mut snapshots = Vec::with_capacity(body.snapshots.len());

    for (index, input) in body.snapshots.into_iter().enumerate() {
        if !seen.insert(input.analysis_id) {
            continue;
        }
        let record = state
            .store
            .get(input.analysis_id)
            .await
            .ok_or_else(|| {
                ApiError::not_found(format!("analysis not found: {}", input.analysis_id))
            })?;
        if record.status != Stage::Complete || record.dna.is_none() {
            return Err(ApiError::conflict(format!(
                "analysis is not complete: {}",
                input.analysis_id
            )));
        }
        let label = {
            let trimmed = input.version_label.trim();
            if trimmed.is_empty() {
                format!("v{}", index + 1)
            } else {
                trimmed.to_string()
            }
        };
        snapshots.push(crate::evolution::EvolutionSnapshotRecord {
            id: Uuid::new_v4(),
            series_id,
            analysis_id: input.analysis_id,
            version_label: label,
            file_name: record.original_name,
            created_at: created_at + chrono::Duration::milliseconds(index as i64),
        });
    }

    if snapshots.is_empty() {
        return Err(ApiError::bad_request("snapshots must not be empty"));
    }

    let name = body
        .name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| format!("series-{}", &series_id.to_string()[..8]));

    let series = crate::evolution::EvolutionSeriesRecord {
        id: series_id,
        name,
        created_at,
        snapshots,
    };
    state.evolution.insert(series.clone()).await;
    Ok(Json(series.into()))
}

pub async fn get_evolution(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<EvolutionSeriesResponse>> {
    let series = state
        .evolution
        .get(id)
        .await
        .ok_or_else(|| ApiError::not_found("evolution series not found"))?;
    Ok(Json(series.into()))
}

pub async fn list_evolution(
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<EvolutionSeriesResponse>>> {
    let series = state.evolution.list(50).await;
    Ok(Json(series.into_iter().map(Into::into).collect()))
}

#[derive(Deserialize)]
pub struct EvolutionGitRequest {
    pub repo: String,
    pub path: String,
    pub max_commits: Option<usize>,
}

pub async fn create_evolution_from_git(
    State(state): State<AppState>,
    Json(body): Json<EvolutionGitRequest>,
) -> ApiResult<Json<EvolutionSeriesResponse>> {
    let repo = crate::git_import::resolve_repo(&state.config.git_repos_dir, &body.repo)?;
    let max_commits = body.max_commits.unwrap_or(8).min(10);
    let history = crate::git_import::read_file_history(&repo, &body.path, max_commits)?;

    let file_stem = std::path::Path::new(&body.path)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("sample.txt");
    let mut snapshot_inputs = Vec::with_capacity(history.len());

    for entry in &history {
        let id = Uuid::new_v4();
        let filename = format!("{file_stem}");
        let storage_key = object_key(id, &filename);
        state.blobs.put_bytes(&storage_key, &entry.bytes).await?;
        let size = entry.bytes.len() as u64;
        let label = if entry.subject.len() > 40 {
            entry.short.clone()
        } else {
            format!("{} · {}", entry.short, entry.subject)
        };
        let record = AnalysisRecord {
            id,
            status: Stage::Queued,
            original_name: filename,
            size_bytes: size,
            mime_type: Some(sniff_mime(file_stem, None)),
            storage_key,
            config: state.config.default_analysis.clone(),
            error: None,
            created_at: Utc::now(),
            completed_at: None,
            dna: None,
            anomalies: Vec::new(),
            progress: Some(analysis_engine::ProgressEvent {
                stage: Stage::Queued,
                progress: 0.0,
                processed_bytes: 0,
                total_bytes: Some(size),
                message: format!("Queued git revision {}", entry.short),
            }),
        };
        state.store.insert(record).await;
        spawn_analysis_job(state.clone(), id);
        wait_analysis_complete(&state, id).await?;
        snapshot_inputs.push(EvolutionSnapshotInput {
            analysis_id: id,
            version_label: label,
        });
    }

    create_evolution(
        State(state),
        Json(CreateEvolutionRequest {
            name: Some(format!("{}:{}", body.repo, body.path)),
            snapshots: snapshot_inputs,
        }),
    )
    .await
}

async fn wait_analysis_complete(state: &AppState, id: Uuid) -> ApiResult<()> {
    for _ in 0..200 {
        tokio::time::sleep(Duration::from_millis(50)).await;
        if let Some(record) = state.store.get(id).await {
            match record.status {
                Stage::Complete => return Ok(()),
                Stage::Failed => {
                    return Err(ApiError::internal(
                        record
                            .error
                            .unwrap_or_else(|| "analysis failed".to_string()),
                    ));
                }
                _ => continue,
            }
        }
    }
    Err(ApiError::internal("timed out waiting for analysis"))
}

#[derive(Deserialize)]
pub struct IsolationExperimentRequest {
    pub analysis_id: Uuid,
}

#[derive(Deserialize)]
pub struct KnnExperimentRequest {
    pub analysis_ids: Vec<Uuid>,
    pub k: Option<usize>,
}

pub async fn experiment_isolation(
    State(state): State<AppState>,
    Json(body): Json<IsolationExperimentRequest>,
) -> ApiResult<Json<analysis_engine::ExperimentResult>> {
    let (_name, dna) = load_completed_dna(&state, body.analysis_id, "analysis").await?;
    Ok(Json(analysis_engine::isolation_score(&dna)))
}

pub async fn experiment_knn_density(
    State(state): State<AppState>,
    Json(body): Json<KnnExperimentRequest>,
) -> ApiResult<Json<analysis_engine::ExperimentResult>> {
    if body.analysis_ids.is_empty() {
        return Err(ApiError::bad_request("analysis_ids must not be empty"));
    }
    if body.analysis_ids.len() > 50 {
        return Err(ApiError::bad_request("analysis_ids capped at 50"));
    }
    let mut dnas = Vec::with_capacity(body.analysis_ids.len());
    let mut labels = Vec::with_capacity(body.analysis_ids.len());
    for id in &body.analysis_ids {
        let (name, dna) = load_completed_dna(&state, *id, "analysis").await?;
        labels.push(name);
        dnas.push(dna);
    }
    let k = body.k.unwrap_or(3);
    Ok(Json(analysis_engine::knn_density(&dnas, &labels, k)))
}

pub async fn not_implemented() -> ApiError {
    ApiError::new(
        axum::http::StatusCode::NOT_IMPLEMENTED,
        "not_implemented",
        "This endpoint is reserved for a later GENOMA phase",
    )
}
