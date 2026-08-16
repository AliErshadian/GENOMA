use feature_engine::extract_features;
use genoma_core::{AnalysisLevel, Chunk};

fn chunk(data: Vec<u8>) -> Chunk {
    Chunk {
        index: 0,
        offset: 0,
        data,
    }
}

#[test]
fn repeated_text_is_low_entropy() {
    let data = b"alpha alpha alpha beta beta gamma\n".repeat(200);
    let features = extract_features(&chunk(data), AnalysisLevel::Balanced);
    assert!(features.entropy_norm < 0.6);
    assert!(features.repetition_score > 0.05);
}

#[test]
fn cycling_bytes_are_diverse() {
    let data: Vec<u8> = (0..=255).cycle().take(8192).collect();
    let features = extract_features(&chunk(data), AnalysisLevel::Deep);
    assert!(features.entropy_norm > 0.95);
    assert!(features.byte_diversity > 0.99);
}

#[test]
fn deep_level_computes_ngrams() {
    let data: Vec<u8> = (0..4096).map(|i| (i % 13) as u8).collect();
    let fast = extract_features(&chunk(data.clone()), AnalysisLevel::Fast);
    let deep = extract_features(&chunk(data), AnalysisLevel::Deep);
    assert_eq!(fast.ngram_score, 0.0);
    assert!(deep.ngram_score > 0.0);
}
