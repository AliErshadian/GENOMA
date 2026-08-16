# Architecture

```text
apps/web          Next.js workspace and Three.js organism
packages/shared-types
crates/genoma-core        config, errors, streaming chunker
crates/pi-engine          PiSource, file dataset, cache, wrap metadata
crates/feature-engine     FAST / BALANCED / DEEP features
crates/dna-engine         raw → π rotation → visual DNA
crates/analysis-engine    pipeline, similarity, anomalies, mutations
crates/api                Axum upload, jobs, SSE, DNA
```

## Data flow

Upload is streamed to a blob store (local `data/uploads` in Phase 1, MinIO/S3-compatible when configured). An analysis job reads that object as a `Read` stream, chunks it, extracts features, applies `dna-v1`, then publishes progress over SSE.

PostgreSQL schema lives in `crates/api/migrations`. Redis and MinIO are provided by `docker-compose.yml` for the full lab. The API still starts without them: jobs stay in memory, blobs stay on disk.

## Memory bound

The chunker holds at most one chunk buffer. Feature structs and DNA vectors are kept; raw chunk bytes are dropped after extraction. A 10 GB file is not materialized as a single `Vec<u8>`.
