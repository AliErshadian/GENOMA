//! Deterministic Digital DNA generation.
//!
//! Digital DNA is a structural representation. It is not a cryptographic hash
//! and is not claimed to be collision resistant.

mod transform;
mod visual;

pub use transform::{pi_offset_for_block, transform_features};
pub use visual::visual_from_vectors;

use feature_engine::ChunkFeatures;
use genoma_core::{
    quantize::clamp01, quantize::quantize, AnalysisConfig, FEATURE_DIM, GENERATOR_VERSION,
};
use pi_engine::{PiSlice, PiSource};
use serde::{Deserialize, Serialize};

pub const DNA_GENERATOR_VERSION: &str = GENERATOR_VERSION;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawFeatureVector {
    pub entropy: f64,
    pub complexity: f64,
    pub repetition: f64,
    pub bit_transition: f64,
    pub compression: f64,
    pub diversity: f64,
    pub values: [f64; FEATURE_DIM],
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PiDerivedVector {
    pub values: [f64; FEATURE_DIM],
    pub pi_offset: u64,
    pub pi_wrapped: bool,
    pub pi_wrap_count: u64,
    pub generator_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VisualDna {
    pub density: f64,
    pub radius: f64,
    pub rotation: f64,
    pub branching: f64,
    pub particle_count: f64,
    pub particle_velocity: f64,
    pub cluster_strength: f64,
    pub noise: f64,
    pub orbital_speed: f64,
    pub geometry_complexity: f64,
    pub hue_mix: f64,
    pub repetition_tint: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChunkDna {
    pub index: u64,
    pub offset: u64,
    pub size: u32,
    pub raw: RawFeatureVector,
    pub pi_derived: PiDerivedVector,
    pub visual: VisualDna,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileDna {
    pub generator_version: String,
    pub pi_base_offset: u64,
    pub chunk_count: u64,
    pub total_bytes: u64,
    pub raw: RawFeatureVector,
    pub pi_derived: PiDerivedVector,
    pub visual: VisualDna,
    pub chunks: Vec<ChunkDna>,
}

impl FileDna {
    pub fn canonical_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

pub fn generate_chunk_dna<S: PiSource>(
    features: &ChunkFeatures,
    source: &S,
    config: &AnalysisConfig,
) -> genoma_core::Result<ChunkDna> {
    let raw_values = features.feature_vector();
    let raw = RawFeatureVector {
        entropy: clamp01(features.entropy_norm),
        complexity: clamp01(features.complexity),
        repetition: clamp01(features.repetition_score),
        bit_transition: clamp01(features.bit_transition_rate),
        compression: clamp01(features.compression_estimate),
        diversity: clamp01(features.byte_diversity),
        values: raw_values,
    };

    let pi_offset = pi_offset_for_block(config.pi_base_offset, features.index);
    let slice: PiSlice = source.get_digits_with_wrap(pi_offset, (FEATURE_DIM * 4) as usize)?;
    let derived_values = transform_features(&raw_values, &slice.digits, features.index);
    let pi_derived = PiDerivedVector {
        values: derived_values,
        pi_offset,
        pi_wrapped: slice.wrapped,
        pi_wrap_count: slice.wrap_count,
        generator_version: DNA_GENERATOR_VERSION.to_string(),
    };
    let visual = visual_from_vectors(&raw, &pi_derived, features.index, features.size);

    Ok(ChunkDna {
        index: features.index,
        offset: features.offset,
        size: features.size,
        raw,
        pi_derived,
        visual,
    })
}

pub fn aggregate_file_dna(chunks: Vec<ChunkDna>, config: &AnalysisConfig) -> FileDna {
    let total_bytes = chunks
        .iter()
        .map(|chunk| u64::from(chunk.size))
        .sum::<u64>()
        .max(1);
    let mut raw_values = [0.0; FEATURE_DIM];
    let mut derived_values = [0.0; FEATURE_DIM];
    let mut entropy = 0.0;
    let mut complexity = 0.0;
    let mut repetition = 0.0;
    let mut bit_transition = 0.0;
    let mut compression = 0.0;
    let mut diversity = 0.0;
    let mut wrapped = false;
    let mut wrap_count = 0;

    for chunk in &chunks {
        let w = f64::from(chunk.size) / total_bytes as f64;
        entropy += chunk.raw.entropy * w;
        complexity += chunk.raw.complexity * w;
        repetition += chunk.raw.repetition * w;
        bit_transition += chunk.raw.bit_transition * w;
        compression += chunk.raw.compression * w;
        diversity += chunk.raw.diversity * w;
        for i in 0..FEATURE_DIM {
            raw_values[i] += chunk.raw.values[i] * w;
            derived_values[i] += chunk.pi_derived.values[i] * w;
        }
        wrapped |= chunk.pi_derived.pi_wrapped;
        wrap_count = wrap_count.max(chunk.pi_derived.pi_wrap_count);
    }

    let raw = RawFeatureVector {
        entropy: clamp01(entropy),
        complexity: clamp01(complexity),
        repetition: clamp01(repetition),
        bit_transition: clamp01(bit_transition),
        compression: clamp01(compression),
        diversity: clamp01(diversity),
        values: raw_values.map(quantize),
    };
    let pi_derived = PiDerivedVector {
        values: derived_values.map(quantize),
        pi_offset: config.pi_base_offset,
        pi_wrapped: wrapped,
        pi_wrap_count: wrap_count,
        generator_version: DNA_GENERATOR_VERSION.to_string(),
    };
    let visual = visual_from_vectors(
        &raw,
        &pi_derived,
        0,
        total_bytes.min(u64::from(u32::MAX)) as u32,
    );

    FileDna {
        generator_version: DNA_GENERATOR_VERSION.to_string(),
        pi_base_offset: config.pi_base_offset,
        chunk_count: chunks.len() as u64,
        total_bytes,
        raw,
        pi_derived,
        visual,
        chunks,
    }
}
