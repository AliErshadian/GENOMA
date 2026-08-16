use genoma_core::quantize::{clamp01, quantize};
use genoma_core::{FEATURE_DIM, PI_STRIDE_DIGITS};
use pi_engine::four_digit_group;

const TAU: f64 = std::f64::consts::TAU;

pub fn pi_offset_for_block(base: u64, block_index: u64) -> u64 {
    base.saturating_add(block_index.saturating_mul(PI_STRIDE_DIGITS))
}

/// π-parameterized Givens rotations. Structure of F is preserved in magnitude;
/// π orients the vector. This is not a cryptographic primitive.
pub fn transform_features(
    features: &[f64; FEATURE_DIM],
    pi_digits: &[u8],
    block_index: u64,
) -> [f64; FEATURE_DIM] {
    let mut vector = *features;
    for k in 0..FEATURE_DIM {
        let start = k * 4;
        let group = if start + 4 <= pi_digits.len() {
            four_digit_group(&pi_digits[start..start + 4])
        } else {
            0
        };
        let theta = TAU * (f64::from(group) / 10_000.0);
        let i = k;
        let j = (k + 1 + (block_index as usize % 3)) % FEATURE_DIM;
        apply_givens(&mut vector, i, j, theta);
    }
    vector.map(|value| clamp01((value + 1.0) * 0.5))
}

fn apply_givens(vector: &mut [f64; FEATURE_DIM], i: usize, j: usize, theta: f64) {
    if i == j {
        return;
    }
    let cos = quantize(theta.cos());
    let sin = quantize(theta.sin());
    let vi = vector[i];
    let vj = vector[j];
    vector[i] = quantize(cos * vi - sin * vj);
    vector[j] = quantize(sin * vi + cos * vj);
}

#[cfg(test)]
mod tests {
    use super::transform_features;
    use genoma_core::FEATURE_DIM;

    #[test]
    fn identical_inputs_match() {
        let f = [
            0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.15, 0.25, 0.35, 0.45, 0.55, 0.65, 0.75, 0.85,
        ];
        let digits = b"1415926535897932384626433832795028841971693993751058209749445923";
        let a = transform_features(&f, digits, 3);
        let b = transform_features(&f, digits, 3);
        assert_eq!(a, b);
        assert_eq!(a.len(), FEATURE_DIM);
    }
}
