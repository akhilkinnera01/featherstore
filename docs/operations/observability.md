# Observability

Featherstore exposes Prometheus metrics at `/metrics` and emits structured logs through Rust `tracing`.

## Metrics endpoint

```bash
curl -fsS http://localhost:8080/metrics
```

Important metrics:

| Metric | Meaning |
| --- | --- |
| `featherstore_http_requests_total{method,path,status}` | HTTP request count by route and status. |
| `featherstore_http_request_duration_seconds_bucket{method,path,status}` | HTTP latency histogram. |
| `featherstore_lookup_requests_total{entity,status}` | Lookup request count by entity and status. |
| `featherstore_lookup_rows_total{entity,found}` | Lookup row count by entity and found/missing status. |
| `featherstore_ingest_rows_total{entity,status}` | Ingested rows by entity and accepted/rejected status. |
| `featherstore_store_rows{entity}` | Current rows in the store by entity. |
| `featherstore_store_schemas` | Current loaded schema count. |
| `featherstore_errors_total{kind}` | Server errors by error kind. |

## Logs

```bash
FEATHERSTORE_LOG_FORMAT=json cargo run -p featherstore-server --release
```

Do not put secrets in feature values or request payloads; logs and metrics should be treated as operational records.

## Grafana

Import `deploy/grafana/featherstore-dashboard.json` as a starter dashboard. Review panels against the metric names emitted by the running server.
