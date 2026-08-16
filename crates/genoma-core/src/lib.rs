//! Core types, errors, configuration, and streaming chunker for GENOMA.
//!
//! GENOMA produces a structural fingerprint ("Digital DNA"), not a cryptographic hash.

pub mod chunk;
pub mod config;
pub mod error;
pub mod ids;
pub mod quantize;

pub use chunk::{Chunk, ChunkSize, Chunker, DEFAULT_CHUNK_SIZE, SUPPORTED_CHUNK_SIZES};
pub use config::{AnalysisConfig, AnalysisLevel};
pub use error::{Error, Result};
pub use ids::AnalysisId;
pub use quantize::quantize;

pub const GENERATOR_VERSION: &str = "dna-v1";
pub const FEATURE_DIM: usize = 16;
pub const PI_STRIDE_DIGITS: u64 = 64;
pub const PRODUCT_NAME: &str = "GENOMA";
pub const TAGLINE: &str = "THE DNA OF DIGITAL DATA";
