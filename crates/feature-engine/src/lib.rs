//! Structural feature extraction.
//!
//! Features describe the internal statistics of a byte chunk. They are not a hash.

mod extract;
mod ngram;
mod stats;

pub use extract::extract_chunk_features;
pub use stats::{byte_histogram, shannon_entropy};

use genoma_core::{quantize::clamp01, quantize::quantize, AnalysisLevel, Chunk, FEATURE_DIM};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChunkFeatures {
    pub index: u64,
    pub offset: u64,
    pub size: u32,
    pub byte_count: u32,
    pub min_byte: u8,
    pub max_byte: u8,
    pub mean_byte: f64,
    pub variance: f64,
    pub entropy_bits: f64,
    pub entropy_norm: f64,
    pub zero_one_ratio: f64,
    pub bit_transition_rate: f64,
    pub average_run_length: f64,
    pub bit_entropy: f64,
    pub repetition_score: f64,
    pub ngram_score: f64,
    pub compression_estimate: f64,
    pub byte_diversity: f64,
    pub complexity: f64,
    pub histogram: Vec<u32>,
}

impl ChunkFeatures {
    pub fn feature_vector(&self) -> [f64; FEATURE_DIM] {
        [
            clamp01(self.entropy_norm),
            clamp01(self.complexity),
            clamp01(self.repetition_score),
            clamp01(self.bit_transition_rate),
            clamp01(self.compression_estimate),
            clamp01(self.byte_diversity),
            clamp01(self.mean_byte / 255.0),
            clamp01((self.variance / (255.0 * 255.0)).sqrt()),
            clamp01(self.zero_one_ratio),
            clamp01(self.bit_entropy),
            clamp01((self.average_run_length / 64.0).min(1.0)),
            clamp01(f64::from(self.min_byte) / 255.0),
            clamp01(f64::from(self.max_byte) / 255.0),
            clamp01(((self.mean_byte - f64::from(self.min_byte)) / 255.0).abs()),
            clamp01(self.ngram_score),
            clamp01((self.entropy_norm * (1.0 - self.repetition_score)).clamp(0.0, 1.0)),
        ]
    }

    pub fn quantized(mut self) -> Self {
        self.mean_byte = quantize(self.mean_byte);
        self.variance = quantize(self.variance);
        self.entropy_bits = quantize(self.entropy_bits);
        self.entropy_norm = clamp01(self.entropy_norm);
        self.zero_one_ratio = clamp01(self.zero_one_ratio);
        self.bit_transition_rate = clamp01(self.bit_transition_rate);
        self.average_run_length = quantize(self.average_run_length);
        self.bit_entropy = clamp01(self.bit_entropy);
        self.repetition_score = clamp01(self.repetition_score);
        self.ngram_score = clamp01(self.ngram_score);
        self.compression_estimate = clamp01(self.compression_estimate);
        self.byte_diversity = clamp01(self.byte_diversity);
        self.complexity = clamp01(self.complexity);
        self
    }
}

pub fn extract_features(chunk: &Chunk, level: AnalysisLevel) -> ChunkFeatures {
    extract_chunk_features(chunk, level).quantized()
}
