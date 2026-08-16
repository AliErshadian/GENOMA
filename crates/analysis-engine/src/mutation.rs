use dna_engine::FileDna;
use genoma_core::quantize::{clamp01, quantize};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Mutation {
    pub chunk_index: u64,
    pub offset: u64,
    pub impact: f64,
    pub confidence: f64,
    pub distance: f64,
}

pub fn detect_mutations(original: &FileDna, current: &FileDna) -> Vec<Mutation> {
    let mut mutations = Vec::new();
    let original_len = original.total_bytes.max(1) as f64;
    let max_index = original.chunks.len().min(current.chunks.len());
    for i in 0..max_index {
        let a = &original.chunks[i];
        let b = &current.chunks[i];
        let distance = l1(&a.raw.values, &b.raw.values);
        if distance < 0.004 {
            continue;
        }
        let impact = (distance * f64::from(b.size) / original_len).min(1.0);
        let neighbor_agreement = neighbor_confidence(original, current, i, distance);
        mutations.push(Mutation {
            chunk_index: b.index,
            offset: b.offset,
            impact: quantize(impact),
            confidence: clamp01(neighbor_agreement),
            distance: quantize(distance),
        });
    }

    if current.chunks.len() != original.chunks.len() {
        let extra = current.chunk_count.abs_diff(original.chunk_count) as f64;
        mutations.push(Mutation {
            chunk_index: current.chunk_count.max(original.chunk_count),
            offset: current.total_bytes.min(original.total_bytes),
            impact: clamp01(extra / original.chunks.len().max(1) as f64),
            confidence: 0.8,
            distance: 1.0,
        });
    }
    mutations.sort_by(|a, b| b.impact.partial_cmp(&a.impact).unwrap());
    mutations
}

fn neighbor_confidence(original: &FileDna, current: &FileDna, index: usize, distance: f64) -> f64 {
    let mut nearby = 0.0;
    let mut n = 0.0;
    let lo = index.saturating_sub(1);
    let hi = (index + 1).min(
        original
            .chunks
            .len()
            .min(current.chunks.len())
            .saturating_sub(1),
    );
    for j in lo..=hi {
        if j == index {
            continue;
        }
        nearby += l1(
            &original.chunks[j].raw.values,
            &current.chunks[j].raw.values,
        );
        n += 1.0;
    }
    let local = if n == 0.0 { 0.0 } else { nearby / n };
    clamp01(0.5 + 0.5 * (distance - local))
}

fn l1(a: &[f64], b: &[f64]) -> f64 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).abs())
        .sum::<f64>()
        / a.len().max(1) as f64
}
