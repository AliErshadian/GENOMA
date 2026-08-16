use std::io::Cursor;

use analysis_engine::{
    analyze_reader, compare_dna, detect_mutations, NoopProgress, SimilarityWeights,
};
use genoma_core::{AnalysisConfig, AnalysisLevel, ChunkSize};
use pi_engine::MemoryPiSource;

const PI_DIGITS: &[u8] =
    b"1415926535897932384626433832795028841971693993751058209749445923078164062862089986280348253421170679\
8214808651328230664709384460955058223172535940812848111745028410270193852110555964462294895493038196\
4428810975665933446128475648233786783165271201909145648566923460348610454326648213393607260249141274";

fn sample_payload() -> Vec<u8> {
    let mut data = Vec::new();
    data.extend(b"GENOMA determinism fixture\n");
    for i in 0..8000_u32 {
        data.push(((i * 17) % 251) as u8);
        if i % 40 == 0 {
            data.extend_from_slice(&[7, 7, 7, 7]);
        }
    }
    data
}

#[test]
fn dna_is_bit_identical_across_three_runs() {
    let source = MemoryPiSource::from_ascii(PI_DIGITS).unwrap();
    let config = AnalysisConfig::default()
        .with_pi_offset(100_000)
        .with_level(AnalysisLevel::Balanced)
        .with_chunk_size(ChunkSize::Kb4);
    let payload = sample_payload();

    let mut jsons = Vec::new();
    for _ in 0..3 {
        let result = analyze_reader(
            Cursor::new(payload.clone()),
            &source,
            &config,
            Some(payload.len() as u64),
            &NoopProgress,
        )
        .unwrap();
        jsons.push(result.dna.canonical_json().unwrap());
    }
    assert_eq!(jsons[0], jsons[1]);
    assert_eq!(jsons[1], jsons[2]);
}

#[test]
fn identical_files_are_similar_mutated_files_are_less() {
    let source = MemoryPiSource::from_ascii(PI_DIGITS).unwrap();
    let config = AnalysisConfig::default().with_chunk_size(ChunkSize::Kb4);
    let original = sample_payload();
    let mut mutated = original.clone();
    for byte in mutated.iter_mut().skip(200).take(300) {
        *byte ^= 0x5A;
    }

    let a = analyze_reader(
        Cursor::new(original.clone()),
        &source,
        &config,
        None,
        &NoopProgress,
    )
    .unwrap();
    let b = analyze_reader(Cursor::new(original), &source, &config, None, &NoopProgress).unwrap();
    let c = analyze_reader(Cursor::new(mutated), &source, &config, None, &NoopProgress).unwrap();

    let same = compare_dna(&a.dna, &b.dna, SimilarityWeights::default());
    let diff = compare_dna(&a.dna, &c.dna, SimilarityWeights::default());
    assert!(same.overall > 0.999);
    assert!(diff.overall < same.overall);

    let mutations = detect_mutations(&a.dna, &c.dna);
    assert!(!mutations.is_empty());
}
