use dna_engine::FileDna;
use genoma_core::quantize::quantize;

use crate::similarity::{compare_dna, SimilarityWeights};

/// Pairs with similarity at or above this get a galaxy link.
pub const GALAXY_LINK_SIMILARITY: f64 = 0.65;

#[derive(Debug, Clone, PartialEq)]
pub struct GalaxyLink {
    pub from: usize,
    pub to: usize,
    pub strength: f64,
}

/// Classical MDS of pairwise DNA distances into 3D (deterministic, no RNG).
pub fn embed_files(dnas: &[FileDna]) -> (Vec<[f64; 3]>, Vec<GalaxyLink>) {
    let n = dnas.len();
    if n == 0 {
        return (Vec::new(), Vec::new());
    }
    if n == 1 {
        return (vec![[0.0, 0.0, 0.0]], Vec::new());
    }

    let mut dist = vec![0.0_f64; n * n];
    let mut links = Vec::new();
    for i in 0..n {
        for j in (i + 1)..n {
            let similarity = compare_dna(&dnas[i], &dnas[j], SimilarityWeights::default()).overall;
            let d = quantize(1.0 - similarity);
            dist[i * n + j] = d;
            dist[j * n + i] = d;
            if similarity >= GALAXY_LINK_SIMILARITY {
                links.push(GalaxyLink {
                    from: i,
                    to: j,
                    strength: quantize(similarity),
                });
            }
        }
    }

    if n == 2 {
        let half = dist[1] * 0.5;
        return (vec![[quantize(-half), 0.0, 0.0], [quantize(half), 0.0, 0.0]], links);
    }

    // Squared distances.
    let mut d2 = vec![0.0_f64; n * n];
    for i in 0..n * n {
        d2[i] = dist[i] * dist[i];
    }

    // Double-center: B = -0.5 * J D² J
    let mut b = vec![0.0_f64; n * n];
    let inv_n = 1.0 / n as f64;
    let mut row_mean = vec![0.0_f64; n];
    let mut col_mean = vec![0.0_f64; n];
    let mut total = 0.0;
    for i in 0..n {
        for j in 0..n {
            let v = d2[i * n + j];
            row_mean[i] += v;
            col_mean[j] += v;
            total += v;
        }
    }
    for i in 0..n {
        row_mean[i] *= inv_n;
        col_mean[i] *= inv_n;
    }
    let grand = total * inv_n * inv_n;
    for i in 0..n {
        for j in 0..n {
            b[i * n + j] = -0.5 * (d2[i * n + j] - row_mean[i] - col_mean[j] + grand);
        }
    }

    let (evals, evecs) = top_eigenpairs(&b, n, 3);
    let mut positions = vec![[0.0_f64; 3]; n];
    for (k, &eval) in evals.iter().enumerate() {
        if eval <= 1e-12 {
            continue;
        }
        let scale = eval.sqrt();
        for i in 0..n {
            positions[i][k] = quantize(evecs[i * 3 + k] * scale);
        }
    }

    // Normalize span so the galaxy fits a unit-ish volume.
    let mut max_abs: f64 = 1e-9;
    for pos in &positions {
        for &c in pos {
            max_abs = max_abs.max(c.abs());
        }
    }
    if max_abs > 1e-9 {
        let inv = 1.8 / max_abs;
        for pos in &mut positions {
            for c in pos.iter_mut() {
                *c = quantize(*c * inv);
            }
        }
    }

    (positions, links)
}

/// Jacobi eigendecomposition; returns the `k` largest eigenpairs (value, column of eigenvectors).
fn top_eigenpairs(matrix: &[f64], n: usize, k: usize) -> (Vec<f64>, Vec<f64>) {
    let mut a = matrix.to_vec();
    let mut v = vec![0.0_f64; n * n];
    for i in 0..n {
        v[i * n + i] = 1.0;
    }

    for _ in 0..(n * n * 8).max(64) {
        let mut p = 0usize;
        let mut q = 1usize;
        let mut max_off = 0.0;
        for i in 0..n {
            for j in (i + 1)..n {
                let val = a[i * n + j].abs();
                if val > max_off {
                    max_off = val;
                    p = i;
                    q = j;
                }
            }
        }
        if max_off < 1e-12 {
            break;
        }

        let app = a[p * n + p];
        let aqq = a[q * n + q];
        let apq = a[p * n + q];
        let theta = 0.5 * (aqq - app).atan2(2.0 * apq);
        // Prefer the numerically stable form:
        let tau = (aqq - app) / (2.0 * apq);
        let t = if apq.abs() < 1e-15 {
            0.0
        } else {
            let sign = if tau >= 0.0 { 1.0 } else { -1.0 };
            sign / (tau.abs() + (1.0 + tau * tau).sqrt())
        };
        let _ = theta;
        let c = 1.0 / (1.0 + t * t).sqrt();
        let s = t * c;

        let app_new = c * c * app - 2.0 * s * c * apq + s * s * aqq;
        let aqq_new = s * s * app + 2.0 * s * c * apq + c * c * aqq;
        a[p * n + p] = app_new;
        a[q * n + q] = aqq_new;
        a[p * n + q] = 0.0;
        a[q * n + p] = 0.0;

        for i in 0..n {
            if i == p || i == q {
                continue;
            }
            let aip = a[i * n + p];
            let aiq = a[i * n + q];
            let new_ip = c * aip - s * aiq;
            let new_iq = s * aip + c * aiq;
            a[i * n + p] = new_ip;
            a[p * n + i] = new_ip;
            a[i * n + q] = new_iq;
            a[q * n + i] = new_iq;
        }

        for i in 0..n {
            let vip = v[i * n + p];
            let viq = v[i * n + q];
            v[i * n + p] = c * vip - s * viq;
            v[i * n + q] = s * vip + c * viq;
        }
    }

    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&i, &j| {
        a[j * n + j]
            .partial_cmp(&a[i * n + i])
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| i.cmp(&j))
    });

    let take = k.min(n);
    let mut evals = Vec::with_capacity(take);
    let mut evecs = vec![0.0_f64; n * take];
    for (slot, &idx) in order.iter().take(take).enumerate() {
        evals.push(a[idx * n + idx].max(0.0));
        // Fix eigenvector sign by first nonzero component for determinism.
        let mut flip = 1.0;
        for i in 0..n {
            let val = v[i * n + idx];
            if val.abs() > 1e-12 {
                if val < 0.0 {
                    flip = -1.0;
                }
                break;
            }
        }
        for i in 0..n {
            evecs[i * take + slot] = v[i * n + idx] * flip;
        }
    }
    (evals, evecs)
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

    fn dist(a: [f64; 3], b: [f64; 3]) -> f64 {
        ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
    }

    #[test]
    fn identical_pair_embeds_near_each_other() {
        let dnas = vec![dummy(0.4), dummy(0.4), dummy(0.95)];
        let (pos, links) = embed_files(&dnas);
        assert_eq!(pos.len(), 3);
        assert!(dist(pos[0], pos[1]) < dist(pos[0], pos[2]));
        assert!(links.iter().any(|link| link.from == 0 && link.to == 1));
    }

    #[test]
    fn embed_is_deterministic() {
        let dnas = vec![dummy(0.2), dummy(0.5), dummy(0.8), dummy(0.2)];
        let a = embed_files(&dnas);
        let b = embed_files(&dnas);
        assert_eq!(a, b);
    }
}
