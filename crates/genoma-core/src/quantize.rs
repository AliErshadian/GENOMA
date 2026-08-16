//! Quantize floating-point values so serialized DNA JSON is stable.

pub const DNA_DECIMALS: i32 = 12;

pub fn quantize(value: f64) -> f64 {
    if !value.is_finite() {
        return 0.0;
    }
    let scale = 10f64.powi(DNA_DECIMALS);
    (value * scale).round() / scale
}

pub fn clamp01(value: f64) -> f64 {
    quantize(value.clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quantize_is_stable() {
        let value = 0.123_456_789_012_345;
        assert_eq!(quantize(value), quantize(value));
        assert_eq!(quantize(f64::NAN), 0.0);
        assert_eq!(quantize(1.0), 1.0);
    }
}
