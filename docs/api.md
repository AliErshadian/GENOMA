# API

Base URL: `http://localhost:8080`

Errors:

```json
{ "error": "message", "code": "bad_request" }
```

## Endpoints

- `GET /api/v1/health`
- `POST /api/v1/analyses` multipart field `file`. Query: `chunk_size`, `level`, `pi_offset`
- `GET /api/v1/analyses/{id}`
- `GET /api/v1/analyses/{id}/progress` SSE
- `POST /api/v1/analyses/demo?file=sample.txt`
- `GET /api/v1/demos`
- `GET /api/v1/dna/{id}`
- `GET /api/v1/anomalies/{id}`
- `POST /api/v1/compare` — 501 (later phase)
- `POST /api/v1/export` — 501
- `GET /api/v1/evolution/{id}` — 501
