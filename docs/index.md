# Featherstore documentation

Featherstore is an experimental MVP feature store for deterministic synthetic recommendation workloads. It pairs a Rust/Axum online serving service with a Python SDK, data generator, CLI tools, benchmark reporting, and Kubernetes packaging.

Current maturity: MVP/experimental. Use it for local evaluation, demos, and trusted internal test environments. Do not treat this release as production-ready.

## What you can do today

- Start the Rust server locally.
- Generate deterministic user, item, and context feature rows.
- Ingest those rows through the Python SDK or CLI.
- Query feature rows through Python or HTTP.
- Export Prometheus metrics and structured logs.
- Build a Docker image and render/install the Helm chart.
- Run smoke benchmarks and produce honest benchmark reports.

## What is not proven

The target `10k QPS at p99 < 5 ms` is a performance target, not a claim. Featherstore only reports that target as proven when a benchmark report records measured request QPS at or above 10,000, p99 below 5 ms, and acceptable error rate on a documented environment.

## Repository map

| Path | Purpose |
| --- | --- |
| `crates/featherstore` | Rust library: API models, store, ingest validation, metrics, snapshots. |
| `crates/featherstore-server` | Rust server binary. |
| `python/src/featherstore` | Python SDK, synthetic generator, CLI, benchmark. |
| `examples/` | Local examples and demo notebook. |
| `deploy/helm/featherstore` | Helm chart. |
| `deploy/grafana/featherstore-dashboard.json` | Starter Grafana dashboard. |
| `docs/` | MkDocs documentation source. |

Start with [Install](install.md), then [Quickstart](quickstart.md).
