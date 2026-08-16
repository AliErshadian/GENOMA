use std::fs::File;
use std::sync::mpsc;
use std::thread;

use analysis_engine::{analyze_reader, ProgressEvent, ProgressSink, Stage};
use chrono::Utc;
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

use crate::state::AppState;

struct ChannelSink {
    tx: mpsc::Sender<ProgressEvent>,
}

impl ProgressSink for ChannelSink {
    fn emit(&self, event: ProgressEvent) {
        let _ = self.tx.send(event);
    }
}

pub fn spawn_analysis_job(state: AppState, analysis_id: Uuid) {
    tokio::spawn(async move {
        if let Err(err) = run_job(state.clone(), analysis_id).await {
            tracing::error!(%analysis_id, error = %err.message, "analysis job failed");
            state
                .store
                .update(analysis_id, |record| {
                    record.status = Stage::Failed;
                    record.error = Some(err.message.clone());
                    record.progress = Some(ProgressEvent {
                        stage: Stage::Failed,
                        progress: 1.0,
                        processed_bytes: record.size_bytes,
                        total_bytes: Some(record.size_bytes),
                        message: err.message.clone(),
                    });
                })
                .await;
            if let Some(record) = state.store.get(analysis_id).await {
                if let Some(event) = record.progress {
                    state.progress.publish(analysis_id, event);
                }
            }
        }
    });
}

async fn run_job(state: AppState, analysis_id: Uuid) -> Result<(), crate::error::ApiError> {
    let record = state
        .store
        .get(analysis_id)
        .await
        .ok_or_else(|| crate::error::ApiError::not_found("analysis not found"))?;
    let path = state.blobs.local_path(&record.storage_key);
    let config = record.config.clone();
    let total = record.size_bytes;
    let pi = state.pi.clone();

    let (sync_tx, sync_rx) = mpsc::channel::<ProgressEvent>();
    let (async_tx, mut async_rx) = tokio::sync::mpsc::unbounded_channel::<ProgressEvent>();
    forward_progress(sync_rx, async_tx);

    let join = tokio::task::spawn_blocking(move || {
        let file = File::open(&path).map_err(genoma_core::Error::from)?;
        analyze_reader(
            file,
            pi.as_ref(),
            &config,
            Some(total),
            &ChannelSink { tx: sync_tx },
        )
    });

    while let Some(event) = async_rx.recv().await {
        let event_clone = event.clone();
        state
            .store
            .update(analysis_id, |record| {
                record.status = event.stage;
                record.progress = Some(event);
            })
            .await;
        state.progress.publish(analysis_id, event_clone);
    }

    let result = join
        .await
        .map_err(|err| crate::error::ApiError::internal(err.to_string()))?
        .map_err(crate::error::ApiError::from)?;

    state
        .store
        .update(analysis_id, |record| {
            record.status = Stage::Complete;
            record.completed_at = Some(Utc::now());
            record.dna = Some(result.dna);
            record.anomalies = result.anomalies;
            record.progress = Some(ProgressEvent {
                stage: Stage::Complete,
                progress: 1.0,
                processed_bytes: result.processed_bytes,
                total_bytes: Some(total),
                message: "GENOMA complete".to_string(),
            });
        })
        .await;
    if let Some(record) = state.store.get(analysis_id).await {
        if let Some(event) = record.progress {
            state.progress.publish(analysis_id, event);
        }
    }
    Ok(())
}

fn forward_progress(
    sync_rx: mpsc::Receiver<ProgressEvent>,
    async_tx: UnboundedSender<ProgressEvent>,
) {
    thread::spawn(move || {
        while let Ok(event) = sync_rx.recv() {
            if async_tx.send(event).is_err() {
                break;
            }
        }
    });
}
