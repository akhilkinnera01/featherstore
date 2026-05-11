# Contributing

Thank you for improving Featherstore. This repository is intentionally small and public-ready.

## Development checks

Run these before sending changes:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python -m pip install -e ./python[dev]
pytest python/tests
```

If Docker and Helm are available, also run:

```bash
docker build -t featherstore:local .
helm lint deploy/helm/featherstore
helm template featherstore deploy/helm/featherstore
```

## Benchmark integrity

Do not commit fake benchmark reports. A proof claim requires measured request QPS >= target, p99 < threshold, and zero errors unless a different error threshold is explicitly documented.

## Security

Do not commit secrets. MVP has no authentication; document trusted-network use only.
