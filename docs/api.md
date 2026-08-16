# API

Base URL: `http://localhost:8080`

Errors:

```json
{ "error": "message", "code": "bad_request" }
```

Progress is emitted by the analysis pipeline. The UI must not invent percentages.

Without PostgreSQL, jobs live in memory (refresh after restart loses them). With `DATABASE_URL`, analyses and DNA survive refresh. With `REDIS_URL`, the latest progress event is cached for reconnects.

## Endpoints

- `GET /api/v1/health`
- `GET /api/v1/analyses` — recent analyses (Explorer)
- `POST /api/v1/analyses` multipart field `file`. Query: `chunk_size`, `level`, `pi_offset`
- `GET /api/v1/analyses/{id}`
- `GET /api/v1/analyses/{id}/progress` SSE
- `GET /api/v1/analyses/{id}/progress/latest` JSON snapshot (`stage`, `progress`, `processed_bytes`)
- `POST /api/v1/analyses/demo?file=sample.txt` also accepts `chunk_size`, `level`
- `GET /api/v1/demos`
- `GET /api/v1/dna/{id}`
- `GET /api/v1/anomalies/{id}` — statistical anomalies for a completed analysis (used by the anomaly workspace panel)
- `POST /api/v1/compare` — body `{ "left_id", "right_id" }`; returns similarity breakdown for two completed analyses (404 if missing, 409 if either incomplete)
- `POST /api/v1/mutations` — body `{ "baseline_id", "current_id" }`; returns chunk-level mutations (404 if missing, 409 if either incomplete)
- `POST /api/v1/galaxy` — body `{ "analysis_ids": Uuid[] }` (1–50, deduped); returns multi-file galaxy nodes with `cluster_id`, classical MDS `position`, and similarity `links` (404 if missing, 409 if any incomplete)
- `GET /api/v1/evolution` — list recent evolution series (cap 50)
- `POST /api/v1/evolution` — body `{ "name"?, "snapshots": [{ "analysis_id", "version_label" }] }` (1–20); creates a series from completed analyses
- `GET /api/v1/evolution/{id}` — evolution series with ordered snapshots (404 if missing)
- `POST /api/v1/evolution/git` — body `{ "repo", "path", "max_commits"? }`; imports commits from an allowlisted repo under `data/repos/` (cap 10), analyzes each revision, returns a series
- `POST /api/v1/experiments/isolation` — body `{ "analysis_id" }`; deterministic isolation path-length heuristic (experimental)
- `POST /api/v1/experiments/knn-density` — body `{ "analysis_ids", "k"? }`; mean distance to k nearest neighbors (experimental)
- `POST /api/v1/auth/register` — body `{ "email", "password" }` (min 8 chars); returns `{ token, user }`
- `POST /api/v1/auth/login` — same body; returns `{ token, user }`
- `POST /api/v1/auth/logout` — revokes current Bearer token
- `GET /api/v1/auth/me` — current user (requires Bearer)
- `POST /api/v1/export` — 501

Auth is **optional by default** (`GENOMA_AUTH_REQUIRED=false`). When `true`, mutating and private routes require `Authorization: Bearer <token>` except health, demos list, register, and login.
