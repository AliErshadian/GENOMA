pub fn sampled_bigram_score(data: &[u8]) -> f64 {
    if data.len() < 4 {
        return 0.0;
    }
    let stride = if data.len() > 64 * 1024 { 8 } else { 1 };
    let mut counts = [0_u32; 256];
    let mut total = 0_u32;
    let mut i = 0;
    while i + 1 < data.len() {
        let bigram = data[i] ^ data[i + 1];
        counts[bigram as usize] += 1;
        total += 1;
        i += stride;
    }
    if total == 0 {
        return 0.0;
    }
    let mut entropy = 0.0;
    for count in counts {
        if count == 0 {
            continue;
        }
        let p = f64::from(count) / f64::from(total);
        entropy -= p * p.log2();
    }
    (1.0 - entropy / 8.0).clamp(0.0, 1.0)
}

pub fn trigram_repeat_score(data: &[u8]) -> f64 {
    if data.len() < 16 {
        return 0.0;
    }
    const BUCKETS: usize = 4096;
    let mut seen = vec![0_u16; BUCKETS];
    let mut repeats = 0_u64;
    let mut total = 0_u64;
    let stride = if data.len() > 256 * 1024 { 4 } else { 1 };
    let mut i = 0;
    while i + 2 < data.len() {
        let key =
            (u32::from(data[i]) << 16) | (u32::from(data[i + 1]) << 8) | u32::from(data[i + 2]);
        let slot = (key as usize) & (BUCKETS - 1);
        if seen[slot] > 0 {
            repeats += 1;
        }
        seen[slot] = seen[slot].saturating_add(1);
        total += 1;
        i += stride;
    }
    if total == 0 {
        0.0
    } else {
        (repeats as f64 / total as f64).clamp(0.0, 1.0)
    }
}
