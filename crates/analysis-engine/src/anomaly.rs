use dna_engine::ChunkDna;
use genoma_core::quantize::{clamp01, quantize};
use serde::{Deserialize, Serialize};

pub const ANOMALY_METHOD: &str = "v1-blend";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Anomaly {
    pub chunk_index: u64,
    pub offset: u64,
    pub score: f64,
    pub entropy_z: f64,
    pub neighbor_distance: f64,
    #[serde(default)]
    pub feature_distance: f64,
    #[serde(default = "default_method")]
    pub method: String,
}

fn default_method() -> String {
    ANOMALY_METHOD.to_string()
}

pub fn detect_anomalies(chunks: &[ChunkDna]) -> Vec<Anomaly> {
    if chunks.len() < 3 {
        return Vec::new();
    }
    let entropies: Vec<f64> = chunks.iter().map(|c| c.raw.entropy).collect();
    let mean = entropies.iter().sum::<f64>() / entropies.len() as f64;
    let var = entropies.iter().map(|e| (e - mean).powi(2)).sum::<f64>() / entropies.len() as f64;
    let std = var.sqrt().max(1e-6);

    let dim = chunks[0].raw.values.len().max(1);
    let mut means = vec![0.0_f64; dim];
    let mut vars = vec![0.0_f64; dim];
    for chunk in chunks {
        for (d, value) in chunk.raw.values.iter().enumerate().take(dim) {
            means[d] += value;
        }
    }
    for mean_d in &mut means {
        *mean_d /= chunks.len() as f64;
    }
    for chunk in chunks {
        for (d, value) in chunk.raw.values.iter().enumerate().take(dim) {
            vars[d] += (value - means[d]).powi(2);
        }
    }
    for var_d in &mut vars {
        *var_d = (*var_d / chunks.len() as f64).max(1e-6);
    }

    let mut anomalies = Vec::new();
    for (i, chunk) in chunks.iter().enumerate() {
        let entropy_z = ((chunk.raw.entropy - mean) / std).abs();
        let neighbor_distance = neighbor_feature_distance(chunks, i);
        let feature_distance = diagonal_mahalanobis(&chunk.raw.values, &means, &vars);
        // Multi-signal blend: entropy outlier + local neighbor + robust feature distance.
        let score = clamp01(
            0.40 * sigmoid(entropy_z - 2.0)
                + 0.30 * neighbor_distance
                + 0.30 * sigmoid(feature_distance - 1.5),
        );
        if score >= 0.35 {
            anomalies.push(Anomaly {
                chunk_index: chunk.index,
                offset: chunk.offset,
                score: quantize(score),
                entropy_z: quantize(entropy_z),
                neighbor_distance: quantize(neighbor_distance),
                feature_distance: quantize(feature_distance),
                method: ANOMALY_METHOD.to_string(),
            });
        }
    }
    anomalies.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    anomalies
}

fn diagonal_mahalanobis(values: &[f64], means: &[f64], vars: &[f64]) -> f64 {
    let n = values.len().min(means.len()).min(vars.len()).max(1) as f64;
    let mut acc = 0.0;
    for i in 0..values.len().min(means.len()).min(vars.len()) {
        let delta = values[i] - means[i];
        acc += (delta * delta) / vars[i];
    }
    (acc / n).sqrt()
}

fn neighbor_feature_distance(chunks: &[ChunkDna], index: usize) -> f64 {
    let current = &chunks[index].raw.values;
    let mut acc = 0.0;
    let mut n = 0.0;
    for j in index.saturating_sub(2)..=(index + 2).min(chunks.len() - 1) {
        if j == index {
            continue;
        }
        acc += l1(current, &chunks[j].raw.values);
        n += 1.0;
    }
    if n == 0.0 {
        0.0
    } else {
        clamp01(acc / n)
    }
}

fn l1(a: &[f64], b: &[f64]) -> f64 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).abs())
        .sum::<f64>()
        / a.len().max(1) as f64
}

fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

#[cfg(test)]
mod tests {
    use super::*;
    use dna_engine::{ChunkDna, PiDerivedVector, RawFeatureVector, VisualDna};
    use genoma_core::{FEATURE_DIM, GENERATOR_VERSION};

    fn chunk(index: u64, entropy: f64, values: [f64; FEATURE_DIM]) -> ChunkDna {
        ChunkDna {
            index,
            offset: index * 1024,
            size: 1024,
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
                pi_offset: index * 64,
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
        }
    }

    #[test]
    fn identical_chunks_produce_no_anomalies() {
        let values = [0.4; FEATURE_DIM];
        let chunks: Vec<_> = (0..5).map(|i| chunk(i, 0.4, values)).collect();
        assert!(detect_anomalies(&chunks).is_empty());
    }

    #[test]
    fn outlier_chunk_ranks_highest() {
        let mut chunks = Vec::new();
        for i in 0..6 {
            let mut values = [0.3; FEATURE_DIM];
            let entropy = if i == 3 {
                values = [0.95; FEATURE_DIM];
                0.95
            } else {
                0.3
            };
            chunks.push(chunk(i, entropy, values));
        }
        let found = detect_anomalies(&chunks);
        assert!(!found.is_empty());
        assert_eq!(found[0].chunk_index, 3);
        assert_eq!(found[0].method, ANOMALY_METHOD);
        assert!(found[0].feature_distance > 0.0);
    }
}
