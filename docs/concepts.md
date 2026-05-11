# Concepts and architecture

## Core concepts

Feature row: a row keyed by `entity` and `id` with named feature values.

Feature schema: an entity-level schema with a version and a list of feature definitions. Each feature has a name, type, and nullability flag.

Entity: the top-level namespace for rows. The synthetic recommendation workload uses `user`, `item`, and `context`.

Ingest: batch loading rows for a single entity schema. Supported modes are `upsert` and `replace_entity`.

Lookup: retrieving one or more rows by entity and id, optionally selecting a subset of feature names.

## Data types

Supported feature data types are `int64`, `float64`, `bool`, `string`, and `null` values in row payloads.

## Architecture

```text
Python SDK, CLI, examples, notebook, benchmark
        | HTTP JSON
        v
Rust Axum server
        |
        v
In-memory feature store
        +-- optional local snapshot file
        +-- Prometheus metrics
        +-- structured tracing logs
```

The hot lookup path is in-memory. HTTP handlers validate requests and call the store. Successful lookups record request and row metrics. Missing keys are represented as successful rows with `found=false`.

## Synthetic recommendation workload

The Python generator creates deterministic rows for:

- `user`: age bucket, country id, and user embedding dimensions.
- `item`: category id, price bucket, and item embedding dimensions.
- `context`: hour bucket, device type, and context embedding dimensions.

The generated values are deterministic for the chosen dimensions and seed.

## Storage and Kubernetes shape

The MVP uses a single-process in-memory store. Optional snapshot load/save supports reproducible local demos and restart checks. The Helm chart deploys a single server pod by default with health/readiness probes and metrics exposure.
