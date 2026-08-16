use std::io::Read;

use dna_engine::{aggregate_file_dna, generate_chunk_dna, ChunkDna, FileDna};
use feature_engine::extract_features;
use genoma_core::{AnalysisConfig, Chunker, Result};
use pi_engine::PiSource;
use serde::{Deserialize, Serialize};

use crate::anomaly::{detect_anomalies, Anomaly};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Stage {
    Queued,
    ReadingFile,
    ExtractingFeatures,
    GeneratingPidna,
    DetectingAnomalies,
    BuildingVisualization,
    Complete,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressEvent {
    pub stage: Stage,
    pub progress: f64,
    pub processed_bytes: u64,
    pub total_bytes: Option<u64>,
    pub message: String,
}

pub trait ProgressSink {
    fn emit(&self, event: ProgressEvent);
}

pub struct NoopProgress;

impl ProgressSink for NoopProgress {
    fn emit(&self, _event: ProgressEvent) {}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub dna: FileDna,
    pub anomalies: Vec<Anomaly>,
    pub processed_bytes: u64,
}

pub fn analyze_reader<R: Read, S: PiSource>(
    reader: R,
    source: &S,
    config: &AnalysisConfig,
    total_bytes: Option<u64>,
    progress: &dyn ProgressSink,
) -> Result<AnalysisResult> {
    progress.emit(ProgressEvent {
        stage: Stage::ReadingFile,
        progress: 0.02,
        processed_bytes: 0,
        total_bytes,
        message: "Reading file".to_string(),
    });

    let mut chunker = Chunker::new(reader, config.chunk_size);
    let mut chunk_dnas: Vec<ChunkDna> = Vec::new();
    let mut processed = 0_u64;
    let mut features_done = 0_u64;

    while let Some(chunk) = chunker.next_chunk()? {
        processed += chunk.len() as u64;
        let read_progress = match total_bytes {
            Some(total) if total > 0 => (processed as f64 / total as f64) * 0.45,
            _ => 0.2,
        };
        progress.emit(ProgressEvent {
            stage: Stage::ExtractingFeatures,
            progress: 0.05 + read_progress.min(0.45),
            processed_bytes: processed,
            total_bytes,
            message: format!("Extracting features for block {}", chunk.index),
        });

        let features = extract_features(&chunk, config.level);
        features_done += 1;

        progress.emit(ProgressEvent {
            stage: Stage::GeneratingPidna,
            progress: 0.55 + (0.25 * (features_done as f64 / (features_done as f64 + 1.0))),
            processed_bytes: processed,
            total_bytes,
            message: format!("Generating πDNA for block {}", chunk.index),
        });

        let dna = generate_chunk_dna(&features, source, config)?;
        chunk_dnas.push(dna);
    }

    progress.emit(ProgressEvent {
        stage: Stage::DetectingAnomalies,
        progress: 0.86,
        processed_bytes: processed,
        total_bytes,
        message: "Detecting statistical anomalies".to_string(),
    });

    let anomalies = detect_anomalies(&chunk_dnas);
    let dna = aggregate_file_dna(chunk_dnas, config);

    progress.emit(ProgressEvent {
        stage: Stage::BuildingVisualization,
        progress: 0.95,
        processed_bytes: processed,
        total_bytes,
        message: "Building visualization parameters".to_string(),
    });

    progress.emit(ProgressEvent {
        stage: Stage::Complete,
        progress: 1.0,
        processed_bytes: processed,
        total_bytes,
        message: "GENOMA complete".to_string(),
    });

    Ok(AnalysisResult {
        dna,
        anomalies,
        processed_bytes: processed,
    })
}
