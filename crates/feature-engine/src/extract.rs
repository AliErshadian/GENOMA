use genoma_core::{AnalysisLevel, Chunk};

use crate::ngram::{sampled_bigram_score, trigram_repeat_score};
use crate::stats::{
    bit_stats, byte_histogram, compression_estimate, repetition_score, shannon_entropy,
    unique_bytes,
};
use crate::ChunkFeatures;

pub fn extract_chunk_features(chunk: &Chunk, level: AnalysisLevel) -> ChunkFeatures {
    let data = &chunk.data;
    let size = data.len() as u32;
    if data.is_empty() {
        return ChunkFeatures {
            index: chunk.index,
            offset: chunk.offset,
            size: 0,
            byte_count: 0,
            min_byte: 0,
            max_byte: 0,
            mean_byte: 0.0,
            variance: 0.0,
            entropy_bits: 0.0,
            entropy_norm: 0.0,
            zero_one_ratio: 0.0,
            bit_transition_rate: 0.0,
            average_run_length: 0.0,
            bit_entropy: 0.0,
            repetition_score: 0.0,
            ngram_score: 0.0,
            compression_estimate: 1.0,
            byte_diversity: 0.0,
            complexity: 0.0,
            histogram: vec![0; 256],
        };
    }

    let hist = byte_histogram(data);
    let entropy_bits = shannon_entropy(&hist, data.len());
    let entropy_norm = entropy_bits / 8.0;
    let diversity = f64::from(unique_bytes(&hist)) / 256.0;
    let bits = bit_stats(data);
    let repetition = repetition_score(data);
    let compression = compression_estimate(data);

    let ngram_score = match level {
        AnalysisLevel::Fast => 0.0,
        AnalysisLevel::Balanced => sampled_bigram_score(data),
        AnalysisLevel::Deep => 0.5 * sampled_bigram_score(data) + 0.5 * trigram_repeat_score(data),
    };

    let mut sum = 0_u64;
    let mut min_byte = u8::MAX;
    let mut max_byte = u8::MIN;
    for &byte in data {
        sum += u64::from(byte);
        min_byte = min_byte.min(byte);
        max_byte = max_byte.max(byte);
    }
    let mean = sum as f64 / data.len() as f64;
    let mut var_acc = 0.0;
    for &byte in data {
        let d = f64::from(byte) - mean;
        var_acc += d * d;
    }
    let variance = var_acc / data.len() as f64;
    let complexity = (0.5 * entropy_norm + 0.3 * diversity + 0.2 * compression).clamp(0.0, 1.0);

    ChunkFeatures {
        index: chunk.index,
        offset: chunk.offset,
        size,
        byte_count: size,
        min_byte,
        max_byte,
        mean_byte: mean,
        variance,
        entropy_bits,
        entropy_norm,
        zero_one_ratio: bits.zero_one_ratio,
        bit_transition_rate: bits.bit_transition_rate,
        average_run_length: bits.average_run_length,
        bit_entropy: bits.bit_entropy,
        repetition_score: repetition,
        ngram_score,
        compression_estimate: compression,
        byte_diversity: diversity,
        complexity,
        histogram: hist.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use genoma_core::{AnalysisLevel, Chunk};

    use super::extract_chunk_features;

    fn chunk(data: Vec<u8>) -> Chunk {
        Chunk {
            index: 0,
            offset: 0,
            data,
        }
    }

    #[test]
    fn repeated_bytes_have_low_entropy() {
        let features = extract_chunk_features(&chunk(vec![7; 4096]), AnalysisLevel::Balanced);
        assert!(features.entropy_norm < 0.05);
        assert!(features.repetition_score > 0.8);
    }

    #[test]
    fn mixed_bytes_have_higher_entropy() {
        let data: Vec<u8> = (0..=255).cycle().take(4096).collect();
        let features = extract_chunk_features(&chunk(data), AnalysisLevel::Fast);
        assert!(features.entropy_norm > 0.9);
        assert!(features.byte_diversity > 0.9);
    }
}
