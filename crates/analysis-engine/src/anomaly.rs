use dna_engine::ChunkDna;
use genoma_core::quantize::{clamp01, quantize};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Anomaly {
    pub chunk_index: u64,
    pub offset: u64,
    pub score: f64,
    pub entropy_z: f64,
    pub neighbor_distance: f64,
}

pub fn detect_anomalies(chunks: &[ChunkDna]) -> Vec<Anomaly> {
    if chunks.len() < 3 {
        return Vec::new();
    }
    let entropies: Vec<f64> = chunks.iter().map(|c| c.raw.entropy).collect();
    let mean = entropies.iter().sum::<f64>() / entropies.len() as f64;
    let var = entropies.iter().map(|e| (e - mean).powi(2)).sum::<f64>() / entropies.len() as f64;
    let std = var.sqrt().max(1e-6);

    let mut anomalies = Vec::new();
    for (i, chunk) in chunks.iter().enumerate() {
        let entropy_z = ((chunk.raw.entropy - mean) / std).abs();
        let neighbor_distance = neighbor_feature_distance(chunks, i);
        let score = clamp01(0.55 * sigmoid(entropy_z - 2.0) + 0.45 * neighbor_distance);
        if score >= 0.35 {
            anomalies.push(Anomaly {
                chunk_index: chunk.index,
                offset: chunk.offset,
                score: quantize(score),
                entropy_z: quantize(entropy_z),
                neighbor_distance: quantize(neighbor_distance),
            });
        }
    }
    anomalies.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    anomalies
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
