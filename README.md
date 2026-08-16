# GENOMA

**THE DNA OF DIGITAL DATA**

> Every file has a structure. We make it visible.

GENOMA is an experimental structural fingerprinting and visualization engine. It streams a digital object, measures its internal statistics, orients those measurements with a deterministic π-derived transform, and renders the result as an interactive 3D mathematical organism.

GENOMA is **not** a replacement for cryptographic hashes such as SHA-256. Digital DNA is a structural representation, not a security primitive.

## What is GENOMA?

GENOMA treats a file as a physical specimen:

1. Stream the bytes (never load a multi-gigabyte file fully into memory).
2. Split it into chunks (default 1 MB).
3. Extract structural features: entropy, bit statistics, repetition, compression behavior.
4. Mix the feature vector with a π segment using Givens rotations (`dna-v1`).
5. Map the result onto visual parameters and render a particle organism.

The visualization is the product. The UI is a scientific instrument around it.

## Why π?

π is not claimed to contain hidden information about files.

GENOMA uses a local π digit dataset as a **deterministic orientation field**. The same file, generator version, π offset, and configuration always produce the same DNA. π does not invent structure; it rotates a measured feature vector so the representation lives in a canonical π-indexed space.

This transform is **not cryptographic**. It is not collision resistant.

## How Digital DNA works

Each chunk has three layers:

| Layer | Meaning |
| --- | --- |
| Raw feature vector | Measured statistics of the bytes |
| π-derived vector | The raw vector after π-parameterized Givens rotations |
| Visual DNA | Documented mapping from those vectors onto density, radius, motion, color |

See [docs/dna-algorithm.md](docs/dna-algorithm.md) and [docs/visualization.md](docs/visualization.md).

## Architecture

```text
File  →  Stream  →  Chunker  →  Feature engine  →  π engine  →  DNA engine
                                                              ↓
Next.js / Three.js  ←  SSE progress  ←  Axum API  ←  analysis pipeline
```

| Piece | Stack |
| --- | --- |
| Engine | Rust workspace (`genoma-core`, `pi-engine`, `feature-engine`, `dna-engine`, `analysis-engine`) |
| API | Axum + Tokio (`crates/api`) |
| Web | Next.js, React, Tailwind, React Three Fiber |
| Data | PostgreSQL (schema ready), Redis (compose), MinIO (compose), local filesystem blobs |

PostgreSQL, Redis, and MinIO are started with Docker for the full lab. The API can run locally with filesystem storage and an in-memory job store when those services are down (local-first path). **Without PostgreSQL, refreshing `/analyze/[id]` after a restart loses the job.**

## Installation

## Installation

Requirements: Rust 1.85+, Node 20+, Python 3.11+ (fixture generation), Docker (optional services).

On Windows without Visual Studio Build Tools, use the GNU/LLVM `rustc` plus LLVM-MinGW on `PATH` (the `x86_64-pc-windows-gnullvm` host). With MSVC, install Visual Studio Build Tools (C++ workload) instead.

```bash
git clone <repo>
cd genoma
copy .env.example .env   # Windows
# cp .env.example .env  # Unix

python scripts/generate_fixtures.py
python scripts/emit_demo_dna.py

docker compose up -d
cargo run -p genoma-api
pnpm install
pnpm --filter @genoma/web dev
```

Open [http://localhost:3000](http://localhost:3000).

Windows PowerShell equivalents are the same commands; use `Copy-Item .env.example .env` if `copy` is ambiguous.

## Development

```bash
cargo test --workspace
cargo fmt --all
cargo clippy --workspace --all-targets
pnpm --filter @genoma/web test
pnpm --filter @genoma/web lint
```

`make up`, `make test`, `make api`, and `make web` wrap the same steps.

## API

| Method | Path | Purpose |
| --- | --- | --- |
| GET | `/api/v1/health` | Service identity |
| GET | `/api/v1/analyses` | Recent analyses |
| POST | `/api/v1/analyses` | Stream upload, start job |
| GET | `/api/v1/analyses/:id` | Status + summary |
| GET | `/api/v1/analyses/:id/progress` | SSE progress |
| GET | `/api/v1/analyses/:id/progress/latest` | JSON progress snapshot |
| POST | `/api/v1/analyses/demo` | Analyze a bundled demo file |
| GET | `/api/v1/dna/:id` | Full Digital DNA |
| GET | `/api/v1/anomalies/:id` | Statistical anomalies |
| POST | `/api/v1/compare` | Similarity between two completed analyses |
| POST | `/api/v1/mutations` | Chunk mutations between baseline and current |
| POST | `/api/v1/galaxy` | Multi-file galaxy nodes from completed analyses |
| GET | `/api/v1/evolution` | List recent evolution series |
| POST | `/api/v1/evolution` | Create evolution series from completed analyses |
| GET | `/api/v1/evolution/:id` | Evolution series with ordered snapshots |
| POST | `/api/v1/evolution/git` | Import allowlisted git file history as a series |
| POST | `/api/v1/export` | Reserved (later phase) |

Progress events are real pipeline stages, not animations:

```json
{
  "stage": "GENERATING_PIDNA",
  "progress": 0.72,
  "processed_bytes": 72819302
}
```

## DNA algorithm

See [docs/dna-algorithm.md](docs/dna-algorithm.md). Generator version `dna-v1` is stored on every fingerprint. Same file + config + π offset ⇒ identical JSON.

## Visualization

See [docs/visualization.md](docs/visualization.md). Particle positions, colors, clustering, and motion are functions of the fingerprint. The scene does not use `Math.random()`.

## Security

Uploaded files are untrusted bytes. GENOMA never executes them, never dynamically imports them, and streams them to object storage / disk. Names are sanitized, extensions are allowlisted, size is capped, and processing is bounded-memory. Authentication is not enabled in Phase 1; the API is structured so it can be added later.

## Privacy

Default posture is temporary local analysis. Cloud retention, if used later, must be explicit. The engine is designed to run entirely on one machine.

## Limitations

- Bundled π dataset is 100,000 decimal digits. Larger offsets wrap; wrap is recorded on the fingerprint.
- Similarity and anomaly scores are heuristic/statistical, not proven metrics.
- Export and auth are roadmap — not faked in the UI.
- DNA identity is deterministic on a given IEEE-754 platform and generator version.

## Roadmap

1. Engine + vertical slice (this release)
2. Hardened API, persistence, progress
3. Immersive 3D organism
4. Comparison, mutation, anomaly workspace — done
5. Galaxy mode / multi-file embedding — done
6. Evolution / Git snapshots — done
7. ML experiments, auth, collaboration

## License

MIT. See [LICENSE](LICENSE) and [docs/DISCLAIMER.md](docs/DISCLAIMER.md).
