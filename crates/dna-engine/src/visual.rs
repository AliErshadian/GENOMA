use genoma_core::quantize::{clamp01, quantize};

use crate::{PiDerivedVector, RawFeatureVector, VisualDna};

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    quantize(a + (b - a) * t.clamp(0.0, 1.0))
}

/// Documented visual encoding. Every field is a deterministic function of
/// raw structural features plus the π-derived orientation vector.
pub fn visual_from_vectors(
    raw: &RawFeatureVector,
    pi_derived: &PiDerivedVector,
    chunk_index: u64,
    size: u32,
) -> VisualDna {
    let entropy = raw.entropy;
    let complexity = raw.complexity;
    let repetition = raw.repetition;
    let motion = raw.bit_transition;
    let orient = pi_derived.values.get(0).copied().unwrap_or(0.0);
    let orient2 = pi_derived.values.get(1).copied().unwrap_or(0.0);
    let size_factor = (f64::from(size).log2() / 20.0).clamp(0.15, 1.0);

    VisualDna {
        density: lerp(0.18, 1.0, entropy),
        radius: lerp(0.55, 2.35, complexity * 0.7 + size_factor * 0.3),
        rotation: quantize(std::f64::consts::TAU * orient + 0.017 * (chunk_index as f64) + orient2),
        branching: lerp(0.12, 1.0, complexity),
        particle_count: lerp(80.0, 1800.0, entropy * 0.75 + complexity * 0.25),
        particle_velocity: lerp(0.02, 0.38, motion),
        cluster_strength: lerp(0.04, 0.92, repetition),
        noise: lerp(0.03, 0.42, (1.0 - repetition) * entropy),
        orbital_speed: lerp(0.03, 0.26, motion * 0.7 + entropy * 0.3),
        geometry_complexity: lerp(0.08, 1.0, complexity),
        hue_mix: clamp01(entropy),
        repetition_tint: clamp01(repetition),
    }
}
