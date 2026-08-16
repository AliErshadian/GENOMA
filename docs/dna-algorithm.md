# DNA algorithm (`dna-v1`)

## Inputs

- File bytes (streamed)
- Chunk size (default 1 MB)
- Analysis level: `FAST`, `BALANCED`, `DEEP`
- π base offset
- Local π digit dataset

## Raw vector (16 dimensions, `[0, 1]`)

Normalized Shannon entropy, complexity, repetition, bit-transition rate, compression estimate, byte diversity, mean, variance, zero/one ratio, bit entropy, run length, min/max, mean-min spread, n-gram score, local structure.

Shannon entropy:

```text
H(X) = -Σ p(x) log2 p(x)
entropy_norm = H / 8
```

Compression estimate is a lightweight rolling-hash match ratio, not a full gzip of the object.

## π transform

For chunk index `i`:

```text
offset = pi_base + i * 64
read 64 decimal digits (16 groups of 4)
θ_k = 2π · (group_k / 10000)
D = R_π(F)   # consecutive Givens rotations
```

`R_π` is orthogonal in exact arithmetic; implementation quantizes sines/cosines to 12 decimal places so JSON is stable. Values are then mapped into `[0, 1]` for the API. This is an orientation of measured structure, not a hash.

If `offset` exceeds the bundled dataset length, digits wrap and `pi_wrapped` is set.

## File DNA

Length-weighted average of chunk vectors, plus the per-chunk list used by the organism.

## Similarity (library)

Weighted blend of entropy closeness, L1 on raw values, repetition/transition closeness, complexity closeness, and cosine on π-derived vectors. Configurable weights. Heuristic.

## Reproducibility

For the same bytes, `dna-v1`, π offset, chunk size, and level, three runs must serialize to identical JSON (quantized floats).
