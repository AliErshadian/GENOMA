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
- `GET /api/v1/anomalies/{id}`
- `POST /api/v1/compare` — 501 (later phase)
- `POST /api/v1/export` — 501
- `GET /api/v1/evolution/{id}` — 501
