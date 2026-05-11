# Featherstore

Featherstore is an experimental, self-hostable feature store MVP for deterministic synthetic recommendation workloads. It combines a Rust/Axum online serving plane, an in-memory feature store with optional local snapshot persistence, batch ingest and point/batch lookup APIs, a Python SDK, synthetic data generator, benchmark client, Docker, Helm, Prometheus metrics, and a starter Grafana dashboard.

Maturity: MVP/experimental. Featherstore is suitable for local evaluation and trusted internal test environments. It is not a production feature store yet.

## What is implemented

- Rust server binary: `featherstore-server`.
- Endpoints: `/healthz`, `/readyz`, `/metrics`, `/v1/ingest/batch`, `/v1/features/lookup`, and `/v1/features/{entity}/{id}`.
- Feature value types: `int64`, `float64`, `bool`, `string`, and `null`.
- Entity schemas with versioned feature definitions.
- Ingest modes: `upsert` and `replace_entity`.
- Missing-key semantics: lookup succeeds with `found=false` when an entity key is absent.
- Python package: `featherstore` with `FeatherstoreClient`, typed models, synthetic dataset helpers, and CLI entry points.
- Benchmark reports with QPS, p50/p95/p99, errors, command, dataset parameters, environment metadata, and proof status.

## Quickstart

Prerequisites: Rust 1.78+, Python 3.10+, and optionally Docker/Helm.

Start the server:

```bash
cargo run -p featherstore-server --release
```

In another terminal:

```bash
python -m pip install -e ./python[dev]
featherstore-load-synthetic   --server http://localhost:8080   --users 1000 --items 1000 --contexts 24   --embedding-dims 4 --batch-size 500
python examples/quickstart.py
```

Manual API check:

```bash
curl -fsS http://localhost:8080/healthz
curl -fsS http://localhost:8080/readyz
curl -fsS 'http://localhost:8080/v1/features/user/42?features=age_bucket,country_id'
```

## Demo notebook and examples

The notebook at `examples/demo_recommendation.ipynb` uses the Python SDK to generate, load, and query a synthetic recommendation dataset.

```bash
python -m pip install -e ./python[dev]
cargo run -p featherstore-server --release
jupyter notebook examples/demo_recommendation.ipynb
```

Equivalent script path:

```bash
python examples/full_recommendation.py
```

## Architecture summary

```text
Python SDK / CLI / notebook / benchmark
        | HTTP JSON
        v
Rust Axum server
        |
        v
Single-process in-memory feature store
        +-- optional local snapshot file
        +-- Prometheus metrics at /metrics
        +-- structured logs via tracing
```

The hot lookup path reads from memory. Snapshot persistence is local reproducibility support, not a production durability layer. Kubernetes deployment currently runs one server pod by default.

## Server configuration

| Variable | Default | Purpose |
| --- | --- | --- |
| `FEATHERSTORE_BIND_ADDR` | `0.0.0.0:8080` | Server bind address. |
| `FEATHERSTORE_LOG_FORMAT` | `compact` | `compact` or `json`. |
| `FEATHERSTORE_MAX_REQUEST_BYTES` | `16777216` | Maximum request body size. |
| `FEATHERSTORE_SNAPSHOT_PATH` | unset | Local snapshot file path. |
| `FEATHERSTORE_PERSIST_SNAPSHOT` | `false` | Save snapshot after ingest when true. |
| `FEATHERSTORE_LOAD_SNAPSHOT_ON_START` | `false` | Load snapshot at startup when true and path exists. |

## API summary

| Method | Path | Description |
| --- | --- | --- |
| `GET` | `/healthz` | Process liveness; returns `{"status":"ok"}`. |
| `GET` | `/readyz` | Store readiness and row/schema counts. |
| `GET` | `/metrics` | Prometheus text exposition. |
| `POST` | `/v1/ingest/batch` | Ingest rows for one entity schema. |
| `POST` | `/v1/features/lookup` | Batch lookup across entities. |
| `GET` | `/v1/features/{entity}/{id}?features=a,b` | Fast single-row lookup. |

Unknown entities/features and malformed ingest requests return HTTP 400. Missing entity ids are not transport errors; they return lookup rows with `found=false`.

## Benchmark status

The performance goal is sustained 10,000 HTTP request QPS with p99 latency below 5 ms on the synthetic recommendation workload.

Current status: not proven. This repository includes the benchmark tool and smoke commands, but it does not claim the target until QA/devops records a benchmark report that meets the threshold on a documented environment.

Smoke benchmark example, not a proof run:

```bash
featherstore-benchmark   --server http://localhost:8080   --scenario single_get_request_qps   --duration 5 --warmup 1 --concurrency 16 --target-qps 1000   --users 1000 --items 1000 --contexts 24 --embedding-dims 4   --output benchmark-smoke.json
```

A real proof run must include duration, warmup, concurrency, target QPS, dataset parameters, exact command, environment metadata, p50/p95/p99, measured request QPS, and error rate. Do not extrapolate entity lookups/sec from batch requests into request QPS.

## Docker

```bash
docker build -t featherstore:local .
docker run --rm -p 8080:8080 featherstore:local
```

## Helm

```bash
helm lint deploy/helm/featherstore
helm template featherstore deploy/helm/featherstore
helm install featherstore deploy/helm/featherstore   --set image.repository=ghcr.io/<github-owner>/featherstore   --set image.tag=<tag-or-sha>
```

The GHCR image convention is `ghcr.io/<github-owner>/featherstore:<tag-or-sha>`. The owner is intentionally not hardcoded.

## Documentation

```bash
python -m pip install -r docs/requirements.txt
python -m mkdocs build --strict
python -m mkdocs serve
```

## Local validation commands

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python -m pip install -e ./python[dev]
pytest python/tests
python -m mkdocs build --strict
python examples/full_recommendation.py  # requires a running server
```

## Limitations

- No authentication, authorization, TLS termination, multi-tenancy, quotas, or billing controls.
- In-memory single-process store; no distributed coordination or high availability.
- Snapshot persistence is local reproducibility support, not a production durability guarantee.
- No advanced point-in-time joins or training-set correctness semantics.
- Kubernetes defaults are for functional validation, not performance proof.
- Benchmark target remains `not proven` until measured honestly on a documented environment.
- API is MVP HTTP/JSON; compatibility is not yet guaranteed across releases.

## License

MIT. See `LICENSE`.
