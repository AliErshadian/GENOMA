use dna_engine::FileDna;
use genoma_core::quantize::quantize;

use crate::similarity::{compare_dna, SimilarityWeights};

/// Average-linkage agglomerative clustering cut when merge distance exceeds this.
pub const CLUSTER_DISTANCE_CUT: f64 = 0.35;

/// Cluster files by structural distance `1 - compare_dna.overall`.
///
/// Returns one cluster id per input index in `0..k`, assigned deterministically
/// (clusters renumbered by first-seen member order).
pub fn cluster_files(dnas: &[FileDna]) -> Vec<u32> {
    let n = dnas.len();
    if n == 0 {
        return Vec::new();
    }
    if n == 1 {
        return vec![0];
    }

    let mut dist = vec![0.0_f64; n * n];
    for i in 0..n {
        for j in (i + 1)..n {
            let d = 1.0
                - compare_dna(&dnas[i], &dnas[j], SimilarityWeights::default()).overall;
            let q = quantize(d);
            dist[i * n + j] = q;
            dist[j * n + i] = q;
        }
    }

    // Each index starts in its own active cluster.
    let mut members: Vec<Vec<usize>> = (0..n).map(|i| vec![i]).collect();
    let mut active: Vec<bool> = vec![true; n];

    loop {
        let mut best_i = None;
        let mut best_j = None;
        let mut best_d = f64::INFINITY;
        for i in 0..n {
            if !active[i] {
                continue;
            }
            for j in (i + 1)..n {
                if !active[j] {
                    continue;
                }
                let d = average_linkage(&members[i], &members[j], &dist, n);
                let better = d < best_d - 1e-12
                    || ((d - best_d).abs() <= 1e-12
                        && match (best_i, best_j) {
                            (Some(bi), Some(bj)) => (i, j) < (bi, bj),
                            _ => true,
                        });
                if better {
                    best_d = d;
                    best_i = Some(i);
                    best_j = Some(j);
                }
            }
        }

        let (Some(i), Some(j)) = (best_i, best_j) else {
            break;
        };
        if best_d > CLUSTER_DISTANCE_CUT {
            break;
        }

        // Merge j into i.
        let mut moved = std::mem::take(&mut members[j]);
        members[i].append(&mut moved);
        members[i].sort_unstable();
        active[j] = false;
    }

    let mut labels = vec![0_u32; n];
    let mut next_id = 0_u32;
    for cluster in members.iter().enumerate().filter(|(idx, _)| active[*idx]) {
        for &member in cluster.1 {
            labels[member] = next_id;
        }
        next_id += 1;
    }
    labels
}

fn average_linkage(a: &[usize], b: &[usize], dist: &[f64], n: usize) -> f64 {
    let mut sum = 0.0;
    let mut count = 0.0;
    for &i in a {
        for &j in b {
            sum += dist[i * n + j];
            count += 1.0;
        }
    }
    if count == 0.0 {
        f64::INFINITY
    } else {
        sum / count
    }
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
    fn identical_files_share_a_cluster() {
        let dnas = vec![dummy(0.4), dummy(0.4), dummy(0.4)];
        let labels = cluster_files(&dnas);
        assert_eq!(labels, vec![0, 0, 0]);
    }

    #[test]
    fn distant_files_split_clusters() {
        let dnas = vec![dummy(0.05), dummy(0.95)];
        let labels = cluster_files(&dnas);
        assert_ne!(labels[0], labels[1]);
        assert_eq!(labels.iter().copied().max().unwrap(), 1);
    }

    #[test]
    fn clustering_is_order_stable_for_same_set() {
        let dnas = vec![dummy(0.1), dummy(0.1), dummy(0.9)];
        let a = cluster_files(&dnas);
        let b = cluster_files(&dnas);
        assert_eq!(a, b);
        assert_eq!(a[0], a[1]);
        assert_ne!(a[0], a[2]);
    }
}
