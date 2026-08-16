//! Classical / statistical ML-style experiments (heuristic, deterministic).

use dna_engine::FileDna;
use genoma_core::quantize::{clamp01, quantize};
use serde::{Deserialize, Serialize};

use crate::similarity::{compare_dna, SimilarityWeights};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExperimentScore {
    pub analysis_index: usize,
    pub label: String,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExperimentResult {
    pub method: String,
    pub description: String,
    pub scores: Vec<ExperimentScore>,
}

/// Deterministic isolation-forest–style path-length score for one DNA fingerprint.
/// Higher score ⇒ more anomalous relative to a synthetic “typical” split forest.
pub fn isolation_score(dna: &FileDna) -> ExperimentResult {
    let values = &dna.raw.values;
    let dim = values.len().max(1);
    let trees = 32_usize;
    let mut path_sum = 0.0;
    for t in 0..trees {
        let mut seed = hash_u64(dna.total_bytes ^ (t as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
        let mut depth = 0_u32;
        let mut lo = 0.0_f64;
        let mut hi = 1.0_f64;
        for _ in 0..16 {
            seed = hash_u64(seed);
            let axis = (seed as usize) % dim;
            seed = hash_u64(seed);
            let split = lo + ((seed as f64 / u64::MAX as f64) * (hi - lo).max(1e-9));
            let point = values[axis];
            depth += 1;
            if point < split {
                hi = split;
            } else {
                lo = split;
            }
            if (hi - lo) < 1e-6 {
                break;
            }
        }
        path_sum += f64::from(depth);
    }
    let avg_path = path_sum / trees as f64;
    // Shorter paths ⇒ more isolated ⇒ higher anomaly score.
    let score = clamp01(1.0 - (avg_path / 16.0));
    ExperimentResult {
        method: "isolation_v1".into(),
        description: "Deterministic isolation path-length heuristic on raw DNA values".into(),
        scores: vec![ExperimentScore {
            analysis_index: 0,
            label: "file".into(),
            score: quantize(score),
        }],
    }
}

/// Average distance to k nearest neighbors using `1 - compare_dna.overall`.
pub fn knn_density(dnas: &[FileDna], labels: &[String], k: usize) -> ExperimentResult {
    let n = dnas.len();
    let k = k.clamp(1, n.saturating_sub(1).max(1));
    let mut scores = Vec::with_capacity(n);
    for i in 0..n {
        let mut distances = Vec::with_capacity(n.saturating_sub(1));
        for j in 0..n {
            if i == j {
                continue;
            }
            let d = 1.0 - compare_dna(&dnas[i], &dnas[j], SimilarityWeights::default()).overall;
            distances.push(d);
        }
        distances.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let take = k.min(distances.len());
        let avg = if take == 0 {
            0.0
        } else {
            distances.iter().take(take).sum::<f64>() / take as f64
        };
        let label = labels
            .get(i)
            .cloned()
            .unwrap_or_else(|| format!("item-{i}"));
        scores.push(ExperimentScore {
            analysis_index: i,
            label,
            score: quantize(clamp01(avg)),
        });
    }
    scores.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    ExperimentResult {
        method: "knn_density_v1".into(),
        description: format!(
            "Mean distance to {k} nearest neighbors (1 - similarity); higher ⇒ sparser"
        ),
        scores,
    }
}

fn hash_u64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
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
    fn isolation_is_deterministic() {
        let dna = dummy(0.7);
        let a = isolation_score(&dna);
        let b = isolation_score(&dna);
        assert_eq!(a, b);
        assert_eq!(a.method, "isolation_v1");
    }

    #[test]
    fn knn_flags_outlier_higher() {
        let dnas = vec![dummy(0.2), dummy(0.21), dummy(0.22), dummy(0.95)];
        let labels = vec!["a".into(), "b".into(), "c".into(), "outlier".into()];
        let result = knn_density(&dnas, &labels, 2);
        assert_eq!(result.scores[0].label, "outlier");
    }
}
