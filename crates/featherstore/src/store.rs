use crate::{
    ingest::validate_ingest_request, FeatureName, FeatureSchema, FeatureValue, IngestBatchRequest,
    IngestMode, IngestSummary, LookupRequest, LookupResponse, LookupRow, StoreError,
};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    time::Instant,
};

pub trait FeatureStore: Send + Sync + 'static {
    fn upsert_batch(&self, req: IngestBatchRequest) -> Result<IngestSummary, StoreError>;
    fn lookup(&self, req: LookupRequest) -> Result<LookupResponse, StoreError>;
    fn get_one(
        &self,
        entity: &str,
        id: &str,
        features: Option<&[String]>,
    ) -> Result<LookupRow, StoreError>;
    fn stats(&self) -> StoreStats;
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct StoreStats {
    pub schemas: usize,
    pub rows: usize,
    pub rows_by_entity: BTreeMap<String, usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoredRow {
    pub values: BTreeMap<FeatureName, FeatureValue>,
    pub event_ts_ms: Option<i64>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StoreState {
    pub schemas: HashMap<String, FeatureSchema>,
    pub rows: HashMap<(String, String), StoredRow>,
}

#[derive(Default)]
pub struct InMemoryFeatureStore {
    inner: RwLock<StoreState>,
}

impl InMemoryFeatureStore {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn from_state(state: StoreState) -> Self {
        Self {
            inner: RwLock::new(state),
        }
    }
    pub fn export_state(&self) -> StoreState {
        self.inner.read().clone()
    }
    pub fn replace_state(&self, state: StoreState) {
        *self.inner.write() = state;
    }
}

impl FeatureStore for InMemoryFeatureStore {
    fn upsert_batch(&self, req: IngestBatchRequest) -> Result<IngestSummary, StoreError> {
        validate_ingest_request(&req)?;
        let mut state = self.inner.write();
        if matches!(req.mode, IngestMode::ReplaceEntity) {
            state
                .rows
                .retain(|(entity, _), _| entity != &req.schema.entity);
        }
        state
            .schemas
            .insert(req.schema.entity.clone(), req.schema.clone());
        for row in &req.rows {
            state.rows.insert(
                (req.schema.entity.clone(), row.id.clone()),
                StoredRow {
                    values: row.values.clone(),
                    event_ts_ms: row.event_ts_ms,
                },
            );
        }
        Ok(IngestSummary {
            entity: req.schema.entity,
            accepted_rows: req.rows.len(),
            rejected_rows: 0,
            total_rows: req.rows.len(),
            schema_version: req.schema.version,
        })
    }

    fn lookup(&self, req: LookupRequest) -> Result<LookupResponse, StoreError> {
        let start = Instant::now();
        let mut rows = Vec::with_capacity(req.requests.len());
        for request in req.requests {
            let row = self.get_one(&request.entity, &request.id, request.features.as_deref())?;
            if row.found || req.include_missing {
                rows.push(row);
            }
        }
        Ok(LookupResponse {
            rows,
            elapsed_us: start.elapsed().as_micros() as u64,
        })
    }

    fn get_one(
        &self,
        entity: &str,
        id: &str,
        features: Option<&[String]>,
    ) -> Result<LookupRow, StoreError> {
        if entity.trim().is_empty() || id.trim().is_empty() {
            return Err(StoreError::BadRequest(
                "entity and id must be non-empty".into(),
            ));
        }
        let state = self.inner.read();
        let schema = state
            .schemas
            .get(entity)
            .ok_or_else(|| StoreError::BadRequest(format!("unknown entity '{}'", entity)))?;
        let stored = match state.rows.get(&(entity.to_string(), id.to_string())) {
            Some(row) => row,
            None => {
                return Ok(LookupRow {
                    entity: entity.to_string(),
                    id: id.to_string(),
                    found: false,
                    values: BTreeMap::new(),
                })
            }
        };
        let values = match features {
            None => stored.values.clone(),
            Some(names) => {
                let schema_names: std::collections::HashSet<_> =
                    schema.features.iter().map(|f| f.name.as_str()).collect();
                let mut out = BTreeMap::new();
                for name in names {
                    if !schema_names.contains(name.as_str()) {
                        return Err(StoreError::BadRequest(format!(
                            "unknown feature '{}' for entity '{}'",
                            name, entity
                        )));
                    }
                    if let Some(value) = stored.values.get(name) {
                        out.insert(name.clone(), value.clone());
                    }
                }
                out
            }
        };
        Ok(LookupRow {
            entity: entity.to_string(),
            id: id.to_string(),
            found: true,
            values,
        })
    }

    fn stats(&self) -> StoreStats {
        let state = self.inner.read();
        let mut rows_by_entity = BTreeMap::new();
        for (entity, _) in state.rows.keys() {
            *rows_by_entity.entry(entity.clone()).or_insert(0) += 1;
        }
        StoreStats {
            schemas: state.schemas.len(),
            rows: state.rows.len(),
            rows_by_entity,
        }
    }
}
