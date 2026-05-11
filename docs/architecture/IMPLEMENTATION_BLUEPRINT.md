# Featherstore Implementation Blueprint

> For the coder task: implement this blueprint in small, testable steps. This document is the contract from architecture to implementation. Do not expand into broad feature-store parity.

Goal: build a minimal Rust online/offline feature store server, Python SDK, deterministic synthetic recommendation workload, benchmark tool, and CI/deployment scaffolding that satisfy `docs/product/PRD.md` and the architecture in `docs/architecture/ARCHITECTURE.md`.

Architecture summary: Rust Axum server exposes health/readiness/metrics, batch ingest, batch lookup, and fast single-row lookup. The hot path is an in-memory feature store. Python SDK owns synthetic data generation, client APIs, and benchmark reporting. Kubernetes/Helm and CI are public-ready but do not overclaim performance.

Tech stack:

- Rust: Tokio, Axum, Serde, tracing, metrics/prometheus, thiserror/anyhow, parking_lot or arc-swap.
- Python: Python 3.10+, httpx or requests, pytest, optional typer/click for CLIs.
- Deployment: Docker, Helm, GitHub Actions, Prometheus/Grafana-compatible observability.

## Global invariants

- No lookup hot path disk I/O.
- Missing entity keys are successful lookup rows with `found=false`.
- Unknown entity/feature/schema/type mismatch is a client error.
- Benchmark proof logic must be impossible to accidentally mark as proven unless thresholds are met.
- Benchmark reports must distinguish HTTP request QPS from entity lookup/sec.
- CI smoke benchmark is not proof.
- No secrets or private absolute paths in committed docs, examples, workflows, or notebooks.

## Task 1: Create repository scaffold

Objective: create the expected multi-language project layout without implementing full logic.

Files:

- Create: `Cargo.toml`
- Create: `crates/featherstore/Cargo.toml`
- Create: `crates/featherstore-server/Cargo.toml`
- Create: `crates/featherstore/src/lib.rs`
- Create: `crates/featherstore-server/src/main.rs`
- Create: `python/pyproject.toml`
- Create: `python/src/featherstore/__init__.py`
- Create: `python/tests/`
- Create: `Dockerfile`
- Create: `.github/workflows/ci.yml`
- Preserve: `docs/product/PRD.md`, `docs/architecture/ARCHITECTURE.md`, `docs/architecture/IMPLEMENTATION_BLUEPRINT.md`

Implementation notes:

- Workspace package name should be `featherstore` where language tooling allows.
- Rust binary name should be `featherstore-server`.
- Python package import name should be `featherstore`.
- Use MIT or Apache-2.0 license unless another repo convention appears before implementation.

Verification:

- `cargo metadata --format-version=1` succeeds.
- `python -m pip install -e ./python` succeeds once minimal package files exist.

## Task 2: Define Rust data model

Objective: implement shared model types and serialization contracts.

Files:

- Create: `crates/featherstore/src/model.rs`
- Modify: `crates/featherstore/src/lib.rs`
- Test: `crates/featherstore/src/model.rs` unit tests or `crates/featherstore/tests/model.rs`

Required public types:

```rust
pub type EntityType = String;
pub type EntityId = String;
pub type FeatureName = String;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct FeatureDef {
    pub name: FeatureName,
    pub dtype: FeatureDType,
    pub nullable: bool,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct FeatureSchema {
    pub entity: EntityType,
    pub version: u32,
    pub features: Vec<FeatureDef>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FeatureDType {
    Int64,
    Float64,
    Bool,
    String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(untagged)]
pub enum FeatureValue {
    Int64(i64),
    Float64(f64),
    Bool(bool),
    String(String),
    Null,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct ApiFeatureRow {
    pub id: EntityId,
    pub values: std::collections::BTreeMap<FeatureName, FeatureValue>,
    pub event_ts_ms: Option<i64>,
}
```

Tests:

- JSON dtype names are snake_case.
- `FeatureValue` serializes/deserializes ints/floats/bools/strings/null.
- Schema feature names are preserved in order.

## Task 3: Define API DTOs and error model

Objective: create request/response DTOs matching the architecture API contract.

Files:

- Create: `crates/featherstore/src/api.rs`
- Create: `crates/featherstore/src/error.rs`
- Modify: `crates/featherstore/src/lib.rs`
- Test: `crates/featherstore/tests/api_json.rs`

Required DTOs:

```rust
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct IngestBatchRequest {
    pub schema: FeatureSchema,
    pub rows: Vec<ApiFeatureRow>,
    #[serde(default = "default_ingest_mode")]
    pub mode: IngestMode,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IngestMode { Upsert, ReplaceEntity }

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct IngestSummary {
    pub entity: EntityType,
    pub accepted_rows: usize,
    pub rejected_rows: usize,
    pub total_rows: usize,
    pub schema_version: u32,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct LookupRequest {
    pub requests: Vec<EntityLookupRequest>,
    #[serde(default = "default_include_missing")]
    pub include_missing: bool,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EntityLookupRequest {
    pub entity: EntityType,
    pub id: EntityId,
    pub features: Option<Vec<FeatureName>>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct LookupResponse {
    pub rows: Vec<LookupRow>,
    pub elapsed_us: u64,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct LookupRow {
    pub entity: EntityType,
    pub id: EntityId,
    pub found: bool,
    pub values: std::collections::BTreeMap<FeatureName, FeatureValue>,
}
```

Error requirements:

- `StoreError::BadRequest(String)` maps to HTTP 400.
- `StoreError::NotReady(String)` maps to HTTP 503.
- `StoreError::Internal(String)` maps to HTTP 500.
- Error response shape: `{"error":"...","kind":"bad_request|not_ready|internal"}`.

Tests:

- Example JSON from architecture deserializes.
- Missing `include_missing` defaults to true.
- Invalid enum strings fail deserialization.

## Task 4: Implement ingest validation

Objective: validate schemas and rows before they enter the store.

Files:

- Create: `crates/featherstore/src/ingest.rs`
- Test: `crates/featherstore/tests/ingest_validation.rs`

Validation rules:

- Entity name must be non-empty ASCII-ish string; do not over-engineer, but reject empty/whitespace.
- Feature names must be non-empty and unique per schema.
- Row IDs must be non-empty.
- Row values must not contain names absent from schema.
- Missing non-null feature values are bad requests.
- Null is allowed only when schema `nullable=true`.
- Type must match dtype; allow `Int64` values where `Float64` is expected only if explicitly converted and documented, otherwise reject for simplicity.

Suggested function:

```rust
pub fn validate_ingest_request(req: &IngestBatchRequest) -> Result<(), StoreError>;
```

Tests:

- Valid user schema/rows pass.
- Duplicate feature names fail.
- Missing required value fails.
- Unknown row value name fails.
- Null non-nullable value fails.
- Wrong value type fails.

## Task 5: Implement in-memory store

Objective: create a correct, simple online store with no hot-path disk I/O.

Files:

- Create: `crates/featherstore/src/store.rs`
- Modify: `crates/featherstore/src/lib.rs`
- Test: `crates/featherstore/tests/store.rs`

Required trait:

```rust
pub trait FeatureStore: Send + Sync + 'static {
    fn upsert_batch(&self, req: IngestBatchRequest) -> Result<IngestSummary, StoreError>;
    fn lookup(&self, req: LookupRequest) -> Result<LookupResponse, StoreError>;
    fn get_one(&self, entity: &str, id: &str, features: Option<&[String]>) -> Result<LookupRow, StoreError>;
    fn stats(&self) -> StoreStats;
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct StoreStats {
    pub schemas: usize,
    pub rows: usize,
    pub rows_by_entity: std::collections::BTreeMap<String, usize>,
}
```

Suggested implementation:

```rust
pub struct InMemoryFeatureStore {
    inner: parking_lot::RwLock<StoreState>,
}

struct StoreState {
    schemas: std::collections::HashMap<String, FeatureSchema>,
    rows: std::collections::HashMap<(String, String), StoredRow>,
}
```

`StoredRow` can initially keep `BTreeMap<String, FeatureValue>` for implementation speed. If benchmark p99 misses target, optimize to schema-indexed vectors in a follow-up commit.

Tests:

- Upsert then lookup all features.
- Upsert then lookup subset.
- Missing key returns `found=false`.
- Unknown feature subset returns bad request.
- Stats are correct.

## Task 6: Build Axum HTTP server

Objective: expose probes, metrics, ingest, lookup, and single-row endpoints.

Files:

- Create: `crates/featherstore/src/server.rs`
- Create: `crates/featherstore/src/config.rs`
- Modify: `crates/featherstore-server/src/main.rs`
- Test: `crates/featherstore/tests/http.rs`

Routes:

- `GET /healthz`
- `GET /readyz`
- `GET /metrics`
- `POST /v1/ingest/batch`
- `POST /v1/features/lookup`
- `GET /v1/features/:entity/:id`

Configuration:

```rust
pub struct ServerConfig {
    pub bind_addr: std::net::SocketAddr,       // default 0.0.0.0:8080
    pub log_format: LogFormat,                // compact or json
    pub max_request_bytes: usize,             // default 16 MiB
    pub snapshot_path: Option<std::path::PathBuf>,
    pub persist_snapshot: bool,
    pub load_snapshot_on_start: bool,
}
```

Environment variables:

- `FEATHERSTORE_BIND_ADDR`
- `FEATHERSTORE_LOG_FORMAT`
- `FEATHERSTORE_MAX_REQUEST_BYTES`
- `FEATHERSTORE_SNAPSHOT_PATH`
- `FEATHERSTORE_PERSIST_SNAPSHOT`
- `FEATHERSTORE_LOAD_SNAPSHOT_ON_START`
- `RUST_LOG`

Handler behavior:

- Health returns immediately if process is alive.
- Readiness returns row/schema counts after store initialization.
- Ingest returns summary and updates metrics.
- Lookup returns response and updates metrics/histograms.
- Single-row endpoint uses query param `features=a,b,c`.

Tests:

- All routes return expected status on happy path.
- Bad ingest returns 400.
- Missing lookup key returns 200 with found false.
- Ready includes stats.

## Task 7: Add metrics and tracing

Objective: make observability available by default.

Files:

- Create: `crates/featherstore/src/metrics.rs`
- Modify: `crates/featherstore/src/server.rs`
- Test: `crates/featherstore/tests/metrics.rs`

Required metrics:

- `featherstore_http_requests_total{method,path,status}`
- `featherstore_http_request_duration_seconds_bucket{method,path,status}`
- `featherstore_lookup_requests_total{entity,status}`
- `featherstore_lookup_rows_total{entity,found}`
- `featherstore_ingest_rows_total{entity,status}`
- `featherstore_store_rows{entity}`
- `featherstore_store_schemas`
- `featherstore_errors_total{kind}`

Implementation notes:

- Prefer a stable Prometheus crate compatible with Axum.
- Keep label cardinality bounded: use route templates, not raw IDs.
- Do not log payload values by default.

Verification:

- After one ingest and lookup, `/metrics` contains request, ingest, lookup, and store metrics.

## Task 8: Add optional snapshot persistence

Objective: support local reproducibility without making production durability claims.

Files:

- Create: `crates/featherstore/src/snapshot.rs`
- Modify: `crates/featherstore/src/store.rs`
- Test: `crates/featherstore/tests/snapshot.rs`

Snapshot requirements:

- Include format version.
- Include schemas and rows.
- Write through temp file and atomic rename.
- Load at startup if configured.
- If load fails, readiness should fail with a clear non-secret error.

Suggested format:

- `serde` + `bincode` or `rmp-serde`.
- Filename is config-driven; do not hardcode user paths.

Tests:

- Round trip preserves rows and schemas.
- Corrupt file returns error.
- Atomic write leaves no partial final file on failure where practical.

## Task 9: Implement Python SDK models and client

Objective: provide ergonomic Python APIs matching Rust HTTP DTOs.

Files:

- Create: `python/src/featherstore/models.py`
- Create: `python/src/featherstore/client.py`
- Modify: `python/src/featherstore/__init__.py`
- Test: `python/tests/test_client_models.py`

Required surface:

```python
class FeatherstoreClient:
    def __init__(self, base_url: str = "http://localhost:8080", timeout: float = 5.0): ...
    def health(self) -> dict: ...
    def ready(self) -> dict: ...
    def ingest_batch(self, schema: FeatureSchema, rows: list[FeatureRow], mode: str = "upsert") -> IngestSummary: ...
    def lookup(self, requests: list[LookupRequest], include_missing: bool = True) -> LookupResponse: ...
    def get_features(self, entity: str, entity_id: str, features: list[str] | None = None) -> LookupRow: ...
```

Model guidance:

- Use dataclasses and conversion helpers to/from JSON dicts.
- Keep dependencies minimal. Use `httpx` or `requests`; choose one and pin a reasonable lower bound.
- Raise a clear `FeatherstoreError` for non-2xx server responses.

Tests:

- Model dict serialization matches API examples.
- Client builds expected URLs and payloads using a mocked transport.
- Non-2xx raises `FeatherstoreError` with status and server error body.

## Task 10: Implement deterministic synthetic recommendation generator

Objective: produce reproducible user/item/context features for ingest and benchmark.

Files:

- Create: `python/src/featherstore/synthetic.py`
- Test: `python/tests/test_synthetic.py`

Required API:

```python
def generate_recommendation_dataset(
    users: int = 100_000,
    items: int = 100_000,
    contexts: int = 24,
    embedding_dims: int = 8,
    seed: int = 13,
) -> SyntheticDataset: ...
```

Feature suggestions:

- User: `age_bucket:int64`, `country_id:int64`, `u_emb_0..N:float64`.
- Item: `category_id:int64`, `price_bucket:int64`, `i_emb_0..N:float64`.
- Context: `hour_bucket:int64`, `device_type:int64`, `c_emb_0..N:float64` or no context embedding if keeping rows small.

Determinism:

- Same parameters and seed must produce identical first/last rows and counts.
- Use Python stdlib `random.Random(seed)` or deterministic arithmetic formulas. Arithmetic formulas are faster and easier to reproduce.

Tests:

- Same seed yields same rows.
- Counts match users/items/contexts.
- Schema feature count matches embedding_dims.
- Batch iterators cover all rows exactly once.

## Task 11: Implement load CLI and quickstart example

Objective: allow users and CI to populate the server from generated data.

Files:

- Create: `python/src/featherstore/cli.py` or separate modules for console scripts
- Create: `examples/quickstart.py`
- Test: CLI smoke where practical

Commands:

```text
featherstore-load-synthetic --server http://localhost:8080 --users 1000 --items 1000 --contexts 24 --embedding-dims 8 --batch-size 5000 --seed 13
```

Behavior:

- Generate schemas/rows.
- Ingest users, items, contexts in batches.
- Print accepted rows and elapsed time.
- Exit non-zero on server errors.

Verification:

- Start server locally.
- Run load command with tiny dataset.
- Query one known row through SDK.

## Task 12: Implement benchmark client and report schema

Objective: measure latency/throughput honestly and produce reusable reports.

Files:

- Create: `python/src/featherstore/benchmark.py`
- Create: `python/src/featherstore/report.py`
- Test: `python/tests/test_benchmark_report.py`

Command:

```text
featherstore-benchmark \
  --server http://localhost:8080 \
  --scenario single_get_request_qps \
  --duration 30 \
  --warmup 5 \
  --concurrency 128 \
  --target-qps 10000 \
  --users 100000 \
  --items 100000 \
  --contexts 24 \
  --embedding-dims 8 \
  --seed 13 \
  --output benchmark-report.json
```

Required scenarios:

- `single_get_request_qps`
- `batch_post_request_qps`
- `recommendation_triple_lookup`
- `sdk_e2e` optional if time permits

Report JSON required fields:

```json
{
  "status": "proven|not_proven|invalid",
  "scenario": "single_get_request_qps",
  "server_url": "http://localhost:8080",
  "command": "...",
  "duration_seconds": 30.0,
  "warmup_seconds": 5.0,
  "target_qps": 10000,
  "measured_qps": 10050.2,
  "entity_lookups_per_second": 10050.2,
  "latency_ms": {"p50": 1.2, "p95": 3.4, "p99": 4.7, "max": 20.1},
  "errors": {"count": 0, "rate": 0.0},
  "dataset": {"users": 100000, "items": 100000, "contexts": 24, "embedding_dims": 8, "seed": 13},
  "environment": {"os": "...", "cpu": "...", "python": "...", "mode": "local|docker|kubernetes|kind|minikube"},
  "proof_threshold": {"qps": 10000, "p99_ms": 5.0},
  "notes": []
}
```

Proof logic:

```python
def proof_status(measured_qps: float, p99_ms: float, error_rate: float, valid: bool) -> str:
    if not valid:
        return "invalid"
    if measured_qps >= 10_000 and p99_ms < 5.0 and error_rate == 0.0:
        return "proven"
    return "not_proven"
```

If implementer chooses to allow tiny non-zero error rates, the report must explicitly state the threshold. Default should be zero errors for proof.

Tests:

- Proof status returns proven only when all thresholds pass.
- Batch scenario reports request QPS and entity lookups/sec separately.
- Invalid too-short duration or unreachable server is `invalid`, not `not_proven`.
- Percentile calculation is correct for known samples.

## Task 13: Add Dockerfile

Objective: produce a runnable server container.

Files:

- Create/modify: `Dockerfile`
- Optional: `.dockerignore`

Requirements:

- Multi-stage build.
- Build Rust release binary.
- Runtime image runs non-root.
- Exposes port `8080`.
- Default command starts `featherstore-server`.
- No secrets copied into image.

Verification:

```text
docker build -t featherstore:local .
docker run --rm -p 8080:8080 featherstore:local
curl -fsS http://localhost:8080/healthz
```

If Docker unavailable locally, leave command in handoff and note exact blocker.

## Task 14: Add Helm chart skeleton

Objective: give DevOps a chart that can be linted and completed/verified.

Files:

- Create: `deploy/helm/featherstore/Chart.yaml`
- Create: `deploy/helm/featherstore/values.yaml`
- Create: `deploy/helm/featherstore/templates/deployment.yaml`
- Create: `deploy/helm/featherstore/templates/service.yaml`
- Create: `deploy/helm/featherstore/templates/servicemonitor.yaml`
- Create: `deploy/helm/featherstore/templates/_helpers.tpl`

Required values:

- `image.repository`
- `image.tag`
- `image.pullPolicy`
- `server.port`
- `server.logFormat`
- `server.snapshot.enabled`
- `resources`
- `service.type`
- `service.port`
- `metrics.enabled`
- `metrics.serviceMonitor.enabled`

Verification:

```text
helm lint deploy/helm/featherstore
helm template featherstore deploy/helm/featherstore
```

## Task 15: Add CI workflows

Objective: create public-ready CI coverage for Rust, Python, docs, Docker, Helm, and benchmark smoke.

Files:

- Create/modify: `.github/workflows/ci.yml`
- Create: `.github/workflows/docker.yml` if separating image build/publish

Required jobs:

- Rust fmt/clippy/test.
- Python install/test.
- Docker build.
- Helm lint/template.
- Benchmark smoke: start server, load tiny dataset, run short benchmark; report should be valid but not proof.

Do not use `|| true` to hide failures. If a tool is optional, gate by availability with explicit skip logic and visible message.

## Task 16: Add observability artifacts

Objective: prepare deployment and docs teams for monitoring.

Files:

- Create: `deploy/grafana/featherstore-dashboard.json`
- Create: `docs/operations/observability.md`

Dashboard panels:

- Request rate.
- Latency p50/p95/p99 if histogram query is available.
- Error rate.
- Ingest rows.
- Store rows by entity.
- Process memory/CPU if available.

Docs should include:

- `curl /metrics` local check.
- Prometheus scrape config or ServiceMonitor values.
- Dashboard import instructions.
- Metrics names and meanings.

## Task 17: Add docs and release notes skeleton

Objective: ensure public repo docs do not lag implementation.

Files:

- Create/modify: `README.md`
- Create: `CONTRIBUTING.md`
- Create: `docs/operations/benchmark.md`
- Create: `docs/operations/kubernetes.md`
- Create: `docs/operations/release.md`
- Optional: docs site config selected by docs owner

Required content:

- Current maturity: MVP/experimental.
- Quickstart local server + load + lookup.
- API summary.
- Python SDK install/use.
- Kubernetes/Helm install.
- Observability.
- Benchmark method and proof semantics.
- Security limitations: no auth in MVP, trusted/internal use only.
- GHCR convention: `ghcr.io/<github-owner>/featherstore:<tag-or-sha>`.

Do not claim the benchmark target is proven until QA hands off measured results.

## Task 18: Final local validation for coder handoff

Objective: provide downstream DevOps/docs/QA with exact status.

Commands to run if tooling is available:

```text
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python -m pip install -e ./python[dev]
pytest python/tests
cargo run -p featherstore-server --release
curl -fsS http://localhost:8080/healthz
featherstore-load-synthetic --server http://localhost:8080 --users 1000 --items 1000 --contexts 24 --embedding-dims 4 --batch-size 500
featherstore-benchmark --server http://localhost:8080 --scenario single_get_request_qps --duration 5 --warmup 1 --concurrency 16 --target-qps 1000 --users 1000 --items 1000 --contexts 24 --embedding-dims 4 --output benchmark-smoke.json
```

Handoff metadata should include:

- Changed files.
- Commands run and pass/fail.
- Any tools unavailable.
- Benchmark smoke status.
- Known performance bottlenecks.

## Interfaces summary for downstream agents

Rust public module exports from `crates/featherstore/src/lib.rs`:

```rust
pub mod api;
pub mod config;
pub mod error;
pub mod ingest;
pub mod metrics;
pub mod model;
pub mod server;
pub mod snapshot;
pub mod store;

pub use api::*;
pub use config::ServerConfig;
pub use error::{ErrorResponse, StoreError};
pub use model::*;
pub use store::{FeatureStore, InMemoryFeatureStore, StoreStats};
```

HTTP endpoints:

```text
GET  /healthz
GET  /readyz
GET  /metrics
POST /v1/ingest/batch
POST /v1/features/lookup
GET  /v1/features/{entity}/{id}?features=a,b,c
```

Python exports from `python/src/featherstore/__init__.py`:

```python
from .client import FeatherstoreClient, FeatherstoreError
from .models import FeatureDef, FeatureSchema, FeatureRow, IngestSummary, LookupRequest, LookupRow, LookupResponse
from .synthetic import SyntheticDataset, generate_recommendation_dataset
```

Console scripts:

```text
featherstore-load-synthetic
featherstore-benchmark
```

## DevOps-specific handoff notes

- Helm chart values must keep image repository/tag configurable.
- `ghcr.io/<github-owner>/featherstore:<tag-or-sha>` is the image convention; discover owner with authenticated GitHub tooling, do not hardcode.
- If no external Kubernetes exists, use kind/minikube fallback and label performance limitations.
- GHCR publish should block only on credentials/permissions, with no token leakage.

## Docs-specific handoff notes

- Use measured benchmark handoff from QA/devops. Until then say `not yet proven`.
- Docs must separate local smoke benchmark from proof benchmark.
- Demo notebook should use small default dataset to run quickly, not the 100k proof dataset.

## QA-specific handoff notes

- Release gate must inspect benchmark report proof logic.
- It is acceptable and honest for final status to be `not proven` if hardware cannot meet target.
- A proof claim requires measured request QPS >= 10,000 and p99 < 5 ms for documented duration/environment.
- Create remediation tasks for release blockers rather than silently weakening acceptance criteria.
