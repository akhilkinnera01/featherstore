# Python SDK

Install from the repository root:

```bash
python -m pip install -e ./python
```

Install development/test dependencies:

```bash
python -m pip install -e ./python[dev]
```

## Client

```python
from featherstore import FeatherstoreClient

with FeatherstoreClient("http://localhost:8080") as client:
    print(client.health())
    print(client.ready())
    row = client.get_features("user", "42", ["age_bucket", "country_id"])
    print(row)
```

Methods:

| Method | Purpose |
| --- | --- |
| `health()` | Calls `GET /healthz`. |
| `ready()` | Calls `GET /readyz`. |
| `ingest_batch(schema, rows, mode="upsert")` | Calls `POST /v1/ingest/batch`. |
| `lookup(requests, include_missing=True)` | Calls `POST /v1/features/lookup`. |
| `get_features(entity, entity_id, features=None)` | Calls `GET /v1/features/{entity}/{id}`. |

Non-2xx responses raise `FeatherstoreError` with the status code and response body.

## Synthetic dataset

```python
from featherstore import generate_recommendation_dataset

dataset = generate_recommendation_dataset(users=1000, items=1000, contexts=24, embedding_dims=4, seed=13)
```

Load it:

```python
for rows in dataset.iter_user_batches(batch_size=500):
    client.ingest_batch(dataset.user_schema, rows)
for rows in dataset.iter_item_batches(batch_size=500):
    client.ingest_batch(dataset.item_schema, rows)
for rows in dataset.iter_context_batches(batch_size=24):
    client.ingest_batch(dataset.context_schema, rows)
```

## CLI tools

```bash
featherstore-load-synthetic --server http://localhost:8080 --users 1000 --items 1000 --contexts 24 --embedding-dims 4 --batch-size 500
featherstore-benchmark --server http://localhost:8080 --scenario recommendation_triple_lookup --duration 5 --warmup 1 --concurrency 16 --target-qps 1000 --users 1000 --items 1000 --contexts 24 --embedding-dims 4 --output benchmark-smoke.json
```

Benchmark scenarios: `single_get_request_qps`, `batch_post_request_qps`, and `recommendation_triple_lookup`. Smoke runs check that the path works; they are not performance proof.
