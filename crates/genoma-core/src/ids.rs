use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AnalysisId(pub Uuid);

impl AnalysisId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(id: Uuid) -> Self {
        Self(id)
    }
}

impl Default for AnalysisId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for AnalysisId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for AnalysisId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl From<AnalysisId> for Uuid {
    fn from(value: AnalysisId) -> Self {
        value.0
    }
}
