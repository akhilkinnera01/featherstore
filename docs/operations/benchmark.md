# Benchmarking

Featherstore includes a Python benchmark client that records measured request throughput, latency percentiles, errors, command line, dataset parameters, and environment metadata.

Benchmark reports are measurement records, not marketing copy.

## Proof standard

The target `10k QPS p99 < 5 ms` is proven only when one benchmark report shows all of the following:

- measured HTTP request QPS is at least 10,000;
- measured p99 latency is below 5 ms;
- error rate is acceptable for the run, ideally zero for the MVP proof;
- duration, warmup, concurrency, dataset size, command, and environment are recorded;
- the result is not extrapolated from entity lookup throughput.

If any condition is missing, the status must remain `not_proven` or `invalid`.

## Load data before benchmarking

```bash
featherstore-load-synthetic   --server http://localhost:8080   --users 100000 --items 100000 --contexts 24   --embedding-dims 8 --batch-size 5000
```

## Smoke run

This confirms the benchmark path works. It is not proof.

```bash
featherstore-benchmark   --server http://localhost:8080   --scenario single_get_request_qps   --duration 5 --warmup 1 --concurrency 16 --target-qps 1000   --users 1000 --items 1000 --contexts 24 --embedding-dims 4   --output benchmark-smoke.json
```

## Proof-style run template

```bash
featherstore-benchmark   --server http://localhost:8080   --scenario single_get_request_qps   --duration 30 --warmup 5 --concurrency 256 --target-qps 10000   --users 100000 --items 100000 --contexts 24 --embedding-dims 8   --mode local   --output benchmark-10k-qps.json
```

## Scenarios

| Scenario | Request type | Meaning |
| --- | --- | --- |
| `single_get_request_qps` | `GET /v1/features/user/{id}` | Measures single-row HTTP GET request QPS. |
| `batch_post_request_qps` | `POST /v1/features/lookup` | Measures batch lookup request QPS with eight user lookups per request. |
| `recommendation_triple_lookup` | `POST /v1/features/lookup` | Looks up one user, one item, and one context row per request. |

Only request QPS counts toward the 10k HTTP request target.
