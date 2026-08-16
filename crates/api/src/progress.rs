use analysis_engine::ProgressEvent;
use tokio::sync::broadcast;
use uuid::Uuid;

#[derive(Clone)]
pub struct ProgressHub {
    tx: broadcast::Sender<(Uuid, ProgressEvent)>,
}

impl ProgressHub {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(256);
        Self { tx }
    }

    pub fn publish(&self, id: Uuid, event: ProgressEvent) {
        let _ = self.tx.send((id, event));
    }

    pub fn subscribe(&self) -> broadcast::Receiver<(Uuid, ProgressEvent)> {
        self.tx.subscribe()
    }
}

impl Default for ProgressHub {
    fn default() -> Self {
        Self::new()
    }
}
