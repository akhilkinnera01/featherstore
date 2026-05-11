# Quickstart

This path starts the server, loads a small deterministic synthetic dataset, and reads features through the SDK and HTTP API.

## 1. Start the server

```bash
cargo run -p featherstore-server --release
```

Expected health response:

```bash
curl -fsS http://localhost:8080/healthz
# {"status":"ok"}
```

## 2. Install the Python SDK

In another terminal:

```bash
python -m pip install -e ./python[dev]
```

## 3. Load synthetic recommendation features

```bash
featherstore-load-synthetic   --server http://localhost:8080   --users 1000   --items 1000   --contexts 24   --embedding-dims 4   --batch-size 500
```

The CLI ingests three entity schemas: `user`, `item`, and `context`.

## 4. Query features

```bash
python examples/quickstart.py
curl -fsS 'http://localhost:8080/v1/features/user/42?features=age_bucket,country_id'
```

Batch lookup:

```bash
curl -fsS http://localhost:8080/v1/features/lookup   -H 'content-type: application/json'   -d '{"requests":[{"entity":"user","id":"42","features":["age_bucket","country_id"]},{"entity":"item","id":"42","features":["category_id","price_bucket"]},{"entity":"context","id":"12","features":["hour_bucket","device_type"]}],"include_missing":true}'
```

## 5. Check readiness and metrics

```bash
curl -fsS http://localhost:8080/readyz
curl -fsS http://localhost:8080/metrics | grep featherstore_
```

## 6. Run the fuller local example

```bash
python examples/full_recommendation.py
```
