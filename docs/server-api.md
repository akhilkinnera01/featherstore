# Server API

Base URL for local development: `http://localhost:8080`.

## Health

`GET /healthz` returns `{"status":"ok"}`.

## Readiness

`GET /readyz` returns store state:

```json
{"status":"ready","schemas":3,"rows":2024,"rows_by_entity":{"user":1000,"item":1000,"context":24}}
```

## Metrics

`GET /metrics` returns Prometheus text exposition.

## Ingest batch

`POST /v1/ingest/batch`

```json
{
  "schema": {"entity":"user","version":1,"features":[{"name":"age_bucket","dtype":"int64","nullable":false}]},
  "rows": [{"id":"42","values":{"age_bucket":5},"event_ts_ms":null}],
  "mode": "upsert"
}
```

Response:

```json
{"entity":"user","accepted_rows":1,"rejected_rows":0,"total_rows":1,"schema_version":1}
```

Modes: `upsert` and `replace_entity`.

## Batch lookup

`POST /v1/features/lookup`

```json
{
  "requests": [
    {"entity":"user","id":"42","features":["age_bucket","country_id"]},
    {"entity":"item","id":"42","features":["category_id"]}
  ],
  "include_missing": true
}
```

Response includes `rows` and `elapsed_us`. If `include_missing` is true, missing keys appear as rows with `found=false`.

## Single-row lookup

`GET /v1/features/{entity}/{id}?features=a,b`

```bash
curl -fsS 'http://localhost:8080/v1/features/user/42?features=age_bucket,country_id'
```

## Error behavior

- Unknown entity schemas and unknown feature names return HTTP 400.
- Malformed JSON returns HTTP 400.
- Bodies above `FEATHERSTORE_MAX_REQUEST_BYTES` are rejected.
- Missing entity ids are not transport errors; they are lookup rows with `found=false`.

## Server environment variables

| Variable | Default | Description |
| --- | --- | --- |
| `FEATHERSTORE_BIND_ADDR` | `0.0.0.0:8080` | Bind address. |
| `FEATHERSTORE_LOG_FORMAT` | `compact` | `compact` or `json`. |
| `FEATHERSTORE_MAX_REQUEST_BYTES` | `16777216` | Request body limit. |
| `FEATHERSTORE_SNAPSHOT_PATH` | unset | Local snapshot file path. |
| `FEATHERSTORE_PERSIST_SNAPSHOT` | `false` | Save a snapshot after ingest. |
| `FEATHERSTORE_LOAD_SNAPSHOT_ON_START` | `false` | Load a snapshot at startup if present. |
