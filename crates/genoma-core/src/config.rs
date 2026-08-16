use serde::{Deserialize, Serialize};

use crate::chunk::{ChunkSize, DEFAULT_CHUNK_SIZE};
use crate::error::{Error, Result};
use crate::GENERATOR_VERSION;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AnalysisLevel {
    Fast,
    Balanced,
    Deep,
}

impl AnalysisLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fast => "FAST",
            Self::Balanced => "BALANCED",
            Self::Deep => "DEEP",
        }
    }
}

impl Default for AnalysisLevel {
    fn default() -> Self {
        Self::Balanced
    }
}

impl std::str::FromStr for AnalysisLevel {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_ascii_uppercase().as_str() {
            "FAST" => Ok(Self::Fast),
            "BALANCED" => Ok(Self::Balanced),
            "DEEP" => Ok(Self::Deep),
            other => Err(Error::config(format!("unknown analysis level: {other}"))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisConfig {
    pub chunk_size: ChunkSize,
    pub level: AnalysisLevel,
    pub pi_base_offset: u64,
    pub generator_version: String,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            chunk_size: DEFAULT_CHUNK_SIZE,
            level: AnalysisLevel::Balanced,
            pi_base_offset: 0,
            generator_version: GENERATOR_VERSION.to_string(),
        }
    }
}

impl AnalysisConfig {
    pub fn with_pi_offset(mut self, offset: u64) -> Self {
        self.pi_base_offset = offset;
        self
    }

    pub fn with_level(mut self, level: AnalysisLevel) -> Self {
        self.level = level;
        self
    }

    pub fn with_chunk_size(mut self, chunk_size: ChunkSize) -> Self {
        self.chunk_size = chunk_size;
        self
    }
}
