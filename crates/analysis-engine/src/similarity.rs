use dna_engine::FileDna;
use genoma_core::quantize::{clamp01, quantize};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SimilarityWeights {
    pub entropy: f64,
    pub distribution: f64,
    pub pattern: f64,
    pub complexity: f64,
    pub vector: f64,
}

impl Default for SimilarityWeights {
    fn default() -> Self {
        Self {
            entropy: 0.18,
            distribution: 0.22,
            pattern: 0.18,
            complexity: 0.18,
            vector: 0.24,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SimilarityBreakdown {
    pub entropy: f64,
    pub distribution: f64,
    pub pattern: f64,
    pub complexity: f64,
    pub overall: f64,
}

pub fn compare_dna(a: &FileDna, b: &FileDna, weights: SimilarityWeights) -> SimilarityBreakdown {
    let entropy = closeness(a.raw.entropy, b.raw.entropy);
    let complexity = closeness(a.raw.complexity, b.raw.complexity);
    let pattern = closeness(a.raw.repetition, b.raw.repetition) * 0.5
        + closeness(a.raw.bit_transition, b.raw.bit_transition) * 0.5;
    let distribution = histogram_closeness(&a.raw.values, &b.raw.values);
    let vector = cosine(&a.pi_derived.values, &b.pi_derived.values);
    let weight_sum = weights.entropy
        + weights.distribution
        + weights.pattern
        + weights.complexity
        + weights.vector;
    let overall = (weights.entropy * entropy
        + weights.distribution * distribution
        + weights.pattern * pattern
        + weights.complexity * complexity
        + weights.vector * vector)
        / weight_sum.max(1e-9);

    SimilarityBreakdown {
        entropy: clamp01(entropy),
        distribution: clamp01(distribution),
        pattern: clamp01(pattern),
        complexity: clamp01(complexity),
        overall: clamp01(overall),
    }
}

fn closeness(a: f64, b: f64) -> f64 {
    quantize(1.0 - (a - b).abs())
}

fn histogram_closeness(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len().min(b.len()).max(1) as f64;
    let mut acc = 0.0;
    for (x, y) in a.iter().zip(b.iter()) {
        acc += (x - y).abs();
    }
    clamp01(1.0 - acc / n)
}

fn cosine(a: &[f64], b: &[f64]) -> f64 {
    let mut dot = 0.0;
    let mut na = 0.0;
    let mut nb = 0.0;
    for (x, y) in a.iter().zip(b.iter()) {
        dot += x * y;
        na += x * x;
        nb += y * y;
    }
    if na == 0.0 || nb == 0.0 {
        return 0.0;
    }
    clamp01((dot / (na.sqrt() * nb.sqrt()) + 1.0) * 0.5)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dna_engine::{FileDna, PiDerivedVector, RawFeatureVector, VisualDna};
    use genoma_core::{FEATURE_DIM, GENERATOR_VERSION};

    fn dummy(entropy: f64) -> FileDna {
        let values = [entropy; FEATURE_DIM];
        FileDna {
            generator_version: GENERATOR_VERSION.to_string(),
            pi_base_offset: 0,
            chunk_count: 1,
            total_bytes: 16,
            raw: RawFeatureVector {
                entropy,
                complexity: entropy,
                repetition: 1.0 - entropy,
                bit_transition: 0.5,
                compression: entropy,
                diversity: entropy,
                values,
            },
            pi_derived: PiDerivedVector {
                values,
                pi_offset: 0,
                pi_wrapped: false,
                pi_wrap_count: 0,
                generator_version: GENERATOR_VERSION.to_string(),
            },
            visual: VisualDna {
                density: entropy,
                radius: 1.0,
                rotation: 0.0,
                branching: 0.5,
                particle_count: 100.0,
                particle_velocity: 0.1,
                cluster_strength: 0.1,
                noise: 0.1,
                orbital_speed: 0.1,
                geometry_complexity: 0.5,
                hue_mix: entropy,
                repetition_tint: 0.1,
            },
            chunks: vec![],
        }
    }

    #[test]
    fn identical_files_are_highly_similar() {
        let a = dummy(0.4);
        let score = compare_dna(&a, &a, SimilarityWeights::default());
        assert!(score.overall > 0.99);
    }

    #[test]
    fn distant_vectors_score_lower() {
        let a = dummy(0.1);
        let b = dummy(0.9);
        let score = compare_dna(&a, &b, SimilarityWeights::default());
        assert!(score.overall < 0.8);
    }
}
