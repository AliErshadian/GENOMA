pub fn byte_histogram(data: &[u8]) -> [u32; 256] {
    let mut hist = [0_u32; 256];
    for &byte in data {
        hist[byte as usize] += 1;
    }
    hist
}

pub fn shannon_entropy(hist: &[u32; 256], len: usize) -> f64 {
    if len == 0 {
        return 0.0;
    }
    let n = len as f64;
    let mut entropy = 0.0;
    for &count in hist {
        if count == 0 {
            continue;
        }
        let p = f64::from(count) / n;
        entropy -= p * p.log2();
    }
    entropy
}

pub fn unique_bytes(hist: &[u32; 256]) -> u32 {
    hist.iter().filter(|count| **count > 0).count() as u32
}

pub fn bit_stats(data: &[u8]) -> BitStats {
    if data.is_empty() {
        return BitStats::default();
    }

    let mut ones = 0_u64;
    let mut transitions = 0_u64;
    let mut runs = 0_u64;
    let mut run_len_sum = 0_u64;
    let mut prev_bit: Option<u8> = None;
    let mut current_run = 0_u64;
    let mut bit_hist = [0_u64; 2];

    for &byte in data {
        for shift in (0..8).rev() {
            let bit = (byte >> shift) & 1;
            ones += u64::from(bit);
            bit_hist[bit as usize] += 1;
            if let Some(prev) = prev_bit {
                if prev != bit {
                    transitions += 1;
                    runs += 1;
                    run_len_sum += current_run;
                    current_run = 1;
                } else {
                    current_run += 1;
                }
            } else {
                current_run = 1;
            }
            prev_bit = Some(bit);
        }
    }
    runs += 1;
    run_len_sum += current_run;

    let total_bits = (data.len() as u64) * 8;
    let zero_one_ratio = ones as f64 / total_bits as f64;
    let bit_transition_rate = transitions as f64 / (total_bits.saturating_sub(1).max(1) as f64);
    let average_run_length = run_len_sum as f64 / runs.max(1) as f64;
    let mut bit_entropy = 0.0;
    for count in bit_hist {
        if count == 0 {
            continue;
        }
        let p = count as f64 / total_bits as f64;
        bit_entropy -= p * p.log2();
    }

    BitStats {
        zero_one_ratio,
        bit_transition_rate,
        average_run_length,
        bit_entropy,
    }
}

#[derive(Debug, Default, Clone)]
pub struct BitStats {
    pub zero_one_ratio: f64,
    pub bit_transition_rate: f64,
    pub average_run_length: f64,
    pub bit_entropy: f64,
}

pub fn repetition_score(data: &[u8]) -> f64 {
    if data.len() < 4 {
        return 0.0;
    }
    let mut runs = 0_u64;
    let mut current = 1_u32;
    let mut max_run = 1_u32;
    for pair in data.windows(2) {
        if pair[0] == pair[1] {
            current += 1;
            max_run = max_run.max(current);
        } else {
            if current >= 3 {
                runs += u64::from(current);
            }
            current = 1;
        }
    }
    if current >= 3 {
        runs += u64::from(current);
    }
    let run_ratio = runs as f64 / data.len() as f64;
    let max_ratio = (f64::from(max_run) / data.len() as f64).min(1.0);
    let sequence = four_byte_repeat_ratio(data);
    (0.4 * run_ratio + 0.2 * max_ratio + 0.4 * sequence).clamp(0.0, 1.0)
}

fn four_byte_repeat_ratio(data: &[u8]) -> f64 {
    if data.len() < 8 {
        return 0.0;
    }
    const BUCKETS: usize = 2048;
    let mut seen = vec![0_u16; BUCKETS];
    let mut repeats = 0_u64;
    let mut total = 0_u64;
    for window in data.windows(4) {
        let key = u32::from_le_bytes([window[0], window[1], window[2], window[3]]);
        let slot = (key as usize) & (BUCKETS - 1);
        if seen[slot] > 0 {
            repeats += 1;
        }
        seen[slot] = seen[slot].saturating_add(1);
        total += 1;
    }
    if total == 0 {
        0.0
    } else {
        (repeats as f64 / total as f64).clamp(0.0, 1.0)
    }
}

/// Lightweight LZ-style match ratio used as a compression estimate.
/// Returns estimated compressed_size / original_size in (0, 1].
pub fn compression_estimate(data: &[u8]) -> f64 {
    if data.len() < 8 {
        return 1.0;
    }
    const WINDOW: usize = 4096;
    let mut table = [u32::MAX; 4096];
    let mut matches = 0_u64;
    let mut i = 0;
    while i + 3 < data.len() {
        let key = u32::from_le_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]);
        let slot = (key as usize) & (table.len() - 1);
        let prev = table[slot];
        if prev != u32::MAX {
            let prev = prev as usize;
            if i - prev <= WINDOW && data.get(prev..prev + 3) == data.get(i..i + 3) {
                matches += 1;
            }
        }
        table[slot] = i as u32;
        i += 1;
    }
    let opportunities = (data.len() - 3) as f64;
    let match_ratio = matches as f64 / opportunities;
    (1.0 - 0.85 * match_ratio).clamp(0.05, 1.0)
}
