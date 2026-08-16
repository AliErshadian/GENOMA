use std::path::PathBuf;

use pi_engine::{FilePiSource, PiSource, EXPECTED_PREFIX};

fn pi_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/pi/pi-digits.bin")
}

#[test]
fn bundled_dataset_prefix_and_range() {
    let path = pi_path();
    if !path.exists() {
        panic!("missing bundled π dataset at {}", path.display());
    }
    let source = FilePiSource::load(&path).unwrap();
    assert!(source.len() >= 100_000);
    assert_eq!(
        source.get_digits(0, EXPECTED_PREFIX.len()).unwrap(),
        EXPECTED_PREFIX
    );
    let mid = source.get_digits(50, 10).unwrap();
    assert_eq!(mid.len(), 10);
    assert!(mid.iter().all(|d| d.is_ascii_digit()));
}

#[test]
fn wrap_is_recorded_past_end() {
    let source = FilePiSource::load(pi_path()).unwrap();
    let slice = source.get_digits_with_wrap(source.len() + 7, 8).unwrap();
    assert!(slice.wrapped);
    assert_eq!(slice.digits, source.get_digits(7, 8).unwrap());
}
