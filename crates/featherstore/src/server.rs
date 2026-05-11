use crate::{
    snapshot, FeatureStore, InMemoryFeatureStore, IngestBatchRequest, LookupRequest, Metrics,
    ServerConfig, StoreError,
};
use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{Response, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;
use std::{sync::Arc, time::Instant};
use tower_http::{limit::RequestBodyLimitLayer, trace::TraceLayer};

#[derive(Clone)]
pub struct AppState {
    pub store: Arc<InMemoryFeatureStore>,
    pub metrics: Metrics,
    pub config: ServerConfig,
}

impl AppState {
    pub fn new(config: ServerConfig) -> Result<Self, StoreError> {
        let store = Arc::new(InMemoryFeatureStore::new());
        if config.load_snapshot_on_start {
            let path = config.snapshot_path.as_ref().ok_or_else(|| {
                StoreError::NotReady(
                    "snapshot load requested without FEATHERSTORE_SNAPSHOT_PATH".into(),
                )
            })?;
            if path.exists() {
                store.replace_state(snapshot::load_snapshot(path)?);
            }
        }
        Ok(Self {
            store,
            metrics: Metrics::default(),
            config,
        })
    }
}

pub fn app(state: AppState) -> Router {
    let limit = state.config.max_request_bytes;
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/metrics", get(metrics))
        .route("/v1/ingest/batch", post(ingest_batch))
        .route("/v1/features/lookup", post(lookup))
        .route("/v1/features/:entity/:id", get(get_features))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(RequestBodyLimitLayer::new(limit))
}

async fn healthz(State(state): State<AppState>) -> impl IntoResponse {
    record_http(&state, "GET", "/healthz", StatusCode::OK, 0.0);
    Json(json!({"status":"ok"}))
}

async fn readyz(State(state): State<AppState>) -> impl IntoResponse {
    let stats = state.store.stats();
    record_store_stats(&state);
    record_http(&state, "GET", "/readyz", StatusCode::OK, 0.0);
    Json(
        json!({"status":"ready","schemas":stats.schemas,"rows":stats.rows,"rows_by_entity":stats.rows_by_entity}),
    )
}

async fn metrics(State(state): State<AppState>) -> impl IntoResponse {
    record_store_stats(&state);
    match state.metrics.render() {
        Ok(body) => Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "text/plain; version=0.0.4")
            .body(Body::from(body))
            .unwrap(),
        Err(err) => StoreError::Internal(format!("render metrics: {err}")).into_response(),
    }
}

async fn ingest_batch(
    State(state): State<AppState>,
    Json(req): Json<IngestBatchRequest>,
) -> impl IntoResponse {
    let start = Instant::now();
    let entity = req.schema.entity.clone();
    match state.store.upsert_batch(req) {
        Ok(summary) => {
            if let Err(err) = persist_if_configured(&state) {
                return record_error_response(&state, "POST", "/v1/ingest/batch", start, err);
            }
            state
                .metrics
                .ingest_rows
                .with_label_values(&[&entity, "accepted"])
                .inc_by(summary.accepted_rows as u64);
            record_store_stats(&state);
            record_http(
                &state,
                "POST",
                "/v1/ingest/batch",
                StatusCode::OK,
                start.elapsed().as_secs_f64(),
            );
            Json(summary).into_response()
        }
        Err(err) => record_error_response(&state, "POST", "/v1/ingest/batch", start, err),
    }
}

async fn lookup(
    State(state): State<AppState>,
    Json(req): Json<LookupRequest>,
) -> impl IntoResponse {
    let start = Instant::now();
    let entities: Vec<String> = req.requests.iter().map(|r| r.entity.clone()).collect();
    match state.store.lookup(req) {
        Ok(response) => {
            for row in &response.rows {
                state
                    .metrics
                    .lookup_rows
                    .with_label_values(&[&row.entity, if row.found { "true" } else { "false" }])
                    .inc();
            }
            for entity in entities {
                state
                    .metrics
                    .lookup_requests
                    .with_label_values(&[&entity, "ok"])
                    .inc();
            }
            record_http(
                &state,
                "POST",
                "/v1/features/lookup",
                StatusCode::OK,
                start.elapsed().as_secs_f64(),
            );
            Json(response).into_response()
        }
        Err(err) => record_error_response(&state, "POST", "/v1/features/lookup", start, err),
    }
}

#[derive(Deserialize)]
struct FeatureQuery {
    features: Option<String>,
}

async fn get_features(
    State(state): State<AppState>,
    Path((entity, id)): Path<(String, String)>,
    Query(query): Query<FeatureQuery>,
) -> impl IntoResponse {
    let start = Instant::now();
    let features = query.features.as_ref().map(|s| {
        s.split(',')
            .filter(|x| !x.is_empty())
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
    });
    match state.store.get_one(&entity, &id, features.as_deref()) {
        Ok(row) => {
            state
                .metrics
                .lookup_requests
                .with_label_values(&[&entity, "ok"])
                .inc();
            state
                .metrics
                .lookup_rows
                .with_label_values(&[&entity, if row.found { "true" } else { "false" }])
                .inc();
            record_http(
                &state,
                "GET",
                "/v1/features/:entity/:id",
                StatusCode::OK,
                start.elapsed().as_secs_f64(),
            );
            Json(row).into_response()
        }
        Err(err) => record_error_response(&state, "GET", "/v1/features/:entity/:id", start, err),
    }
}

fn persist_if_configured(state: &AppState) -> Result<(), StoreError> {
    if state.config.persist_snapshot {
        let path = state.config.snapshot_path.as_ref().ok_or_else(|| {
            StoreError::Internal(
                "snapshot persist requested without FEATHERSTORE_SNAPSHOT_PATH".into(),
            )
        })?;
        snapshot::save_snapshot(path, &state.store.export_state())?;
    }
    Ok(())
}

fn record_error_response(
    state: &AppState,
    method: &str,
    path: &str,
    start: Instant,
    err: StoreError,
) -> axum::response::Response {
    state.metrics.errors.with_label_values(&[err.kind()]).inc();
    record_http(
        state,
        method,
        path,
        err.status(),
        start.elapsed().as_secs_f64(),
    );
    err.into_response()
}

fn record_http(state: &AppState, method: &str, path: &str, status: StatusCode, seconds: f64) {
    let status_s = status.as_u16().to_string();
    state
        .metrics
        .http_requests
        .with_label_values(&[method, path, &status_s])
        .inc();
    state
        .metrics
        .http_duration
        .with_label_values(&[method, path, &status_s])
        .observe(seconds);
}

fn record_store_stats(state: &AppState) {
    let stats = state.store.stats();
    state.metrics.store_schemas.set(stats.schemas as i64);
    for (entity, rows) in stats.rows_by_entity {
        state
            .metrics
            .store_rows
            .with_label_values(&[&entity])
            .set(rows as i64);
    }
}
