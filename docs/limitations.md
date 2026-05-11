# Limitations

Featherstore is an MVP. Keep these limitations visible in README, docs, demos, and release notes.

## Security

- No authentication or authorization.
- No TLS termination in the server.
- No multi-tenancy, quotas, or per-client isolation.
- Intended for trusted local or internal environments only.

## Storage and reliability

- The online store is in-memory and single-process.
- No replication, failover, or distributed coordination.
- Optional snapshots support local reproducibility, not production durability.
- Restart behavior depends on snapshot configuration.

## Feature-store semantics

- No advanced point-in-time joins for training sets.
- No transformation DSL.
- No registry service beyond schemas loaded into the running server.
- No compatibility guarantee for the MVP HTTP API.

## Performance

- `10k QPS p99 < 5 ms` is a target, not a claim.
- Kubernetes defaults are functional validation defaults, not performance tuning.
- Local kind/minikube results must be labeled local fallback.
- Batch entity lookup throughput must not be presented as HTTP request QPS.

## Operations

- Helm chart deploys one server pod by default.
- Grafana dashboard is a starter asset.
- CI workflows are repository validation, not a release certification process.
