#!/usr/bin/env python3
"""Emit landing-page DNA JSON from a real demo file using the dna-v1 mapping."""

from __future__ import annotations

import json
import math
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PI = ROOT / "data" / "pi" / "pi-digits.bin"
DEMO = ROOT / "data" / "demos" / "sample.txt"
OUT = ROOT / "apps" / "web" / "public" / "demo-dna.json"
FEATURE_DIM = 16
DECIMALS = 12
TAU = math.tau


def quantize(value: float) -> float:
    if not math.isfinite(value):
        return 0.0
    scale = 10**DECIMALS
    return round(value * scale) / scale


def clamp01(value: float) -> float:
    return quantize(min(1.0, max(0.0, value)))


def shannon(hist: list[int], n: int) -> float:
    if n == 0:
        return 0.0
    entropy = 0.0
    for count in hist:
        if not count:
            continue
        p = count / n
        entropy -= p * math.log2(p)
    return entropy


def bit_stats(data: bytes) -> tuple[float, float, float, float]:
    ones = 0
    transitions = 0
    runs = 0
    run_sum = 0
    prev = None
    current = 0
    bit_hist = [0, 0]
    for byte in data:
        for shift in range(7, -1, -1):
            bit = (byte >> shift) & 1
            ones += bit
            bit_hist[bit] += 1
            if prev is None:
                current = 1
            elif prev != bit:
                transitions += 1
                runs += 1
                run_sum += current
                current = 1
            else:
                current += 1
            prev = bit
    runs += 1
    run_sum += current
    total = len(data) * 8
    zero_one = ones / total
    trans = transitions / max(total - 1, 1)
    avg_run = run_sum / max(runs, 1)
    bit_entropy = 0.0
    for count in bit_hist:
        if not count:
            continue
        p = count / total
        bit_entropy -= p * math.log2(p)
    return zero_one, trans, avg_run, bit_entropy


def repetition_score(data: bytes) -> float:
    if len(data) < 4:
        return 0.0
    runs = 0
    current = 1
    max_run = 1
    for a, b in zip(data, data[1:]):
        if a == b:
            current += 1
            max_run = max(max_run, current)
        else:
            if current >= 3:
                runs += current
            current = 1
    if current >= 3:
        runs += current
    run_ratio = runs / len(data)
    max_ratio = min(max_run / len(data), 1.0)
    return min(1.0, max(0.0, 0.7 * run_ratio + 0.3 * max_ratio))


def compression_estimate(data: bytes) -> float:
    if len(data) < 8:
        return 1.0
    table = [2**32 - 1] * 4096
    matches = 0
    i = 0
    window = 4096
    while i + 3 < len(data):
        key = int.from_bytes(data[i : i + 4], "little")
        slot = key & 4095
        prev = table[slot]
        if prev != 2**32 - 1 and i - prev <= window and data[prev : prev + 3] == data[i : i + 3]:
            matches += 1
        table[slot] = i
        i += 1
    match_ratio = matches / (len(data) - 3)
    return min(1.0, max(0.05, 1.0 - 0.85 * match_ratio))


def sampled_bigram(data: bytes) -> float:
    if len(data) < 4:
        return 0.0
    counts = [0] * 256
    total = 0
    i = 0
    while i + 1 < len(data):
        counts[data[i] ^ data[i + 1]] += 1
        total += 1
        i += 1
    entropy = 0.0
    for count in counts:
        if not count:
            continue
        p = count / total
        entropy -= p * math.log2(p)
    return min(1.0, max(0.0, 1.0 - entropy / 8.0))


def extract(data: bytes, index: int, offset: int) -> dict:
    hist = [0] * 256
    for byte in data:
        hist[byte] += 1
    n = len(data)
    entropy_bits = shannon(hist, n)
    entropy_norm = entropy_bits / 8.0
    diversity = sum(1 for count in hist if count) / 256.0
    zero_one, trans, avg_run, bit_entropy = bit_stats(data)
    repetition = repetition_score(data)
    compression = compression_estimate(data)
    ngram = sampled_bigram(data)
    mean = sum(data) / n
    variance = sum((b - mean) ** 2 for b in data) / n
    complexity = min(1.0, max(0.0, 0.5 * entropy_norm + 0.3 * diversity + 0.2 * compression))
    values = [
        clamp01(entropy_norm),
        clamp01(complexity),
        clamp01(repetition),
        clamp01(trans),
        clamp01(compression),
        clamp01(diversity),
        clamp01(mean / 255.0),
        clamp01(math.sqrt(variance / (255.0 * 255.0))),
        clamp01(zero_one),
        clamp01(bit_entropy),
        clamp01(min(avg_run / 64.0, 1.0)),
        clamp01(min(data) / 255.0),
        clamp01(max(data) / 255.0),
        clamp01(abs(mean - min(data)) / 255.0),
        clamp01(ngram),
        clamp01(entropy_norm * (1.0 - repetition)),
    ]
    raw = {
        "entropy": clamp01(entropy_norm),
        "complexity": clamp01(complexity),
        "repetition": clamp01(repetition),
        "bit_transition": clamp01(trans),
        "compression": clamp01(compression),
        "diversity": clamp01(diversity),
        "values": values,
    }
    return {
        "index": index,
        "offset": offset,
        "size": n,
        "raw": raw,
        "average_run_length": avg_run,
    }


def transform(features: list[float], digits: bytes, block_index: int) -> list[float]:
    vector = list(features)
    for k in range(FEATURE_DIM):
        group = int(digits[k * 4 : k * 4 + 4].decode())
        theta = TAU * (group / 10000.0)
        i = k
        j = (k + 1 + (block_index % 3)) % FEATURE_DIM
        cos = quantize(math.cos(theta))
        sin = quantize(math.sin(theta))
        vi, vj = vector[i], vector[j]
        vector[i] = quantize(cos * vi - sin * vj)
        vector[j] = quantize(sin * vi + cos * vj)
    return [clamp01((value + 1.0) * 0.5) for value in vector]


def visual(raw: dict, derived: list[float], chunk_index: int, size: int) -> dict:
    entropy = raw["entropy"]
    complexity = raw["complexity"]
    repetition = raw["repetition"]
    motion = raw["bit_transition"]
    orient = derived[0]
    orient2 = derived[1]
    size_factor = min(1.0, max(0.15, math.log2(max(size, 2)) / 20.0))

    def lerp(a: float, b: float, t: float) -> float:
        return quantize(a + (b - a) * min(1.0, max(0.0, t)))

    return {
        "density": lerp(0.18, 1.0, entropy),
        "radius": lerp(0.55, 2.35, complexity * 0.7 + size_factor * 0.3),
        "rotation": quantize(TAU * orient + 0.017 * chunk_index + orient2),
        "branching": lerp(0.12, 1.0, complexity),
        "particle_count": lerp(80.0, 1800.0, entropy * 0.75 + complexity * 0.25),
        "particle_velocity": lerp(0.02, 0.38, motion),
        "cluster_strength": lerp(0.04, 0.92, repetition),
        "noise": lerp(0.03, 0.42, (1.0 - repetition) * entropy),
        "orbital_speed": lerp(0.03, 0.26, motion * 0.7 + entropy * 0.3),
        "geometry_complexity": lerp(0.08, 1.0, complexity),
        "hue_mix": clamp01(entropy),
        "repetition_tint": clamp01(repetition),
    }


def main() -> None:
    data = DEMO.read_bytes()
    pi = PI.read_bytes()
    extracted = extract(data, 0, 0)
    offset = 0
    digits = bytes(pi[(offset + i) % len(pi)] for i in range(FEATURE_DIM * 4))
    derived = transform(extracted["raw"]["values"], digits, 0)
    vis = visual(extracted["raw"], derived, 0, extracted["size"])
    chunk = {
        "index": 0,
        "offset": 0,
        "size": extracted["size"],
        "raw": extracted["raw"],
        "pi_derived": {
            "values": derived,
            "pi_offset": offset,
            "pi_wrapped": False,
            "pi_wrap_count": 0,
            "generator_version": "dna-v1",
        },
        "visual": vis,
    }
    dna = {
        "generator_version": "dna-v1",
        "pi_base_offset": 0,
        "chunk_count": 1,
        "total_bytes": extracted["size"],
        "raw": extracted["raw"],
        "pi_derived": chunk["pi_derived"],
        "visual": vis,
        "chunks": [chunk],
    }
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(dna, indent=2) + "\n", encoding="utf-8")
    print(f"wrote {OUT}")


if __name__ == "__main__":
    main()
