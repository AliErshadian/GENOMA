#!/usr/bin/env python3
"""Generate bundled π digits and safe demo files. No network access."""

from __future__ import annotations

import argparse
import csv
import json
import math
import struct
import zlib
from decimal import Decimal, getcontext
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PI_PATH = ROOT / "data" / "pi" / "pi-digits.bin"
DEMO_DIR = ROOT / "data" / "demos"
DIGIT_COUNT = 100_000
PI_PREFIX = "14159265358979323846"


def generate_pi_digits(count: int = DIGIT_COUNT) -> bytes:
    """Chudnovsky series: ~14 decimal digits per term."""
    extra = 16
    getcontext().prec = count + extra
    c = 426880 * Decimal(10005).sqrt()
    m = Decimal(1)
    l = Decimal(13591409)
    x = Decimal(1)
    k = Decimal(6)
    s = l
    terms = count // 14 + 2
    for i in range(1, terms):
        m *= (k * k * k - 16 * k) / (i * i * i)
        l += 545140134
        x *= -262537412640768000
        s += (m * l) / x
        k += 12
    pi = c / s
    text = format(pi, "f")
    if not text.startswith("3."):
        raise RuntimeError(f"unexpected pi representation: {text[:16]!r}")
    digits = text[2 : 2 + count]
    if len(digits) != count:
        raise RuntimeError("failed to generate requested digit count")
    if not digits.startswith(PI_PREFIX):
        raise RuntimeError(f"pi prefix mismatch: {digits[:20]}")
    return digits.encode("ascii")


def write_pi() -> None:
    PI_PATH.parent.mkdir(parents=True, exist_ok=True)
    digits = generate_pi_digits()
    PI_PATH.write_bytes(digits)
    print(f"wrote {len(digits)} pi digits to {PI_PATH}")


def lcg_bytes(n: int, seed: int = 0xC0FFEE) -> bytes:
    x = seed & 0xFFFFFFFF
    out = bytearray(n)
    for i in range(n):
        x = (1_664_525 * x + 1_013_904_223) & 0xFFFFFFFF
        out[i] = (x >> 16) & 0xFF
    return bytes(out)


def write_png(path: Path, width: int, height: int) -> None:
    def chunk(tag: bytes, data: bytes) -> bytes:
        return (
            struct.pack(">I", len(data))
            + tag
            + data
            + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)
        )

    raw = bytearray()
    for y in range(height):
        raw.append(0)
        for x in range(width):
            t = (x * 13 + y * 7) & 0xFF
            raw.extend((t, (255 - t) // 2, (x * y) & 0xFF))
    ihdr = struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0)
    png = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", ihdr) + chunk(b"IDAT", zlib.compress(bytes(raw), 9)) + chunk(b"IEND", b"")
    path.write_bytes(png)


def write_demos() -> None:
    DEMO_DIR.mkdir(parents=True, exist_ok=True)

    (DEMO_DIR / "sample.txt").write_text(
        "GENOMA demo text.\n"
        "Repeated structure: alpha alpha alpha beta beta gamma.\n"
        "A short prose paragraph with natural language entropy, used to contrast\n"
        "highly structured JSON and CSV samples against a binary blob.\n"
        * 12,
        encoding="utf-8",
    )

    payload = {
        "project": "GENOMA",
        "kind": "demo",
        "layers": ["raw", "pi-derived", "visual"],
        "blocks": [{"id": i, "label": f"block-{i}", "values": [i % 7, i % 11, i % 13]} for i in range(40)],
    }
    (DEMO_DIR / "sample.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    with (DEMO_DIR / "sample.csv").open("w", newline="", encoding="utf-8") as handle:
        writer = csv.writer(handle)
        writer.writerow(["index", "entropy", "complexity", "repetition"])
        for i in range(200):
            writer.writerow(
                [
                    i,
                    f"{0.3 + 0.5 * math.sin(i / 9):.6f}",
                    f"{0.4 + 0.4 * math.cos(i / 13):.6f}",
                    f"{0.05 + 0.02 * (i % 5):.6f}",
                ]
            )

    (DEMO_DIR / "sample.bin").write_bytes(lcg_bytes(64 * 1024, seed=0x47454E4F))
    write_png(DEMO_DIR / "sample-image.png", 48, 48)
    print(f"wrote demo files to {DEMO_DIR}")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--pi", action="store_true")
    parser.add_argument("--demos", action="store_true")
    args = parser.parse_args()
    if not args.pi and not args.demos:
        args.pi = True
        args.demos = True
    if args.pi:
        write_pi()
    if args.demos:
        write_demos()


if __name__ == "__main__":
    main()
