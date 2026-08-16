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
- `POST /api/v1/export` — 501
- `GET /api/v1/evolution/{id}` — 501
