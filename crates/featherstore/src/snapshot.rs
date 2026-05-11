use crate::{
    store::{StoreState, StoredRow},
    FeatureSchema, StoreError,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::Path};

const SNAPSHOT_FORMAT_VERSION: u32 = 1;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoreSnapshot {
    pub format_version: u32,
    pub state: SerializableStoreState,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SerializableStoreState {
    pub schemas: Vec<FeatureSchema>,
    pub rows: Vec<SerializableRow>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SerializableRow {
    pub entity: String,
    pub id: String,
    pub row: StoredRow,
}

impl From<&StoreState> for SerializableStoreState {
    fn from(state: &StoreState) -> Self {
        Self {
            schemas: state.schemas.values().cloned().collect(),
            rows: state
                .rows
                .iter()
                .map(|((entity, id), row)| SerializableRow {
                    entity: entity.clone(),
                    id: id.clone(),
                    row: row.clone(),
                })
                .collect(),
        }
    }
}

impl From<SerializableStoreState> for StoreState {
    fn from(value: SerializableStoreState) -> Self {
        let schemas = value
            .schemas
            .into_iter()
            .map(|schema| (schema.entity.clone(), schema))
            .collect();
        let rows: HashMap<(String, String), StoredRow> = value
            .rows
            .into_iter()
            .map(|row| ((row.entity, row.id), row.row))
            .collect();
        StoreState { schemas, rows }
    }
}

pub fn save_snapshot(path: &Path, state: &StoreState) -> Result<(), StoreError> {
    let snapshot = StoreSnapshot {
        format_version: SNAPSHOT_FORMAT_VERSION,
        state: SerializableStoreState::from(state),
    };
    let bytes = serde_json::to_vec(&snapshot)
        .map_err(|e| StoreError::Internal(format!("serialize snapshot: {e}")))?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| StoreError::Internal(format!("create snapshot directory: {e}")))?;
    }
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, bytes)
        .map_err(|e| StoreError::Internal(format!("write snapshot temp file: {e}")))?;
    fs::rename(&tmp, path)
        .map_err(|e| StoreError::Internal(format!("rename snapshot temp file: {e}")))?;
    Ok(())
}

pub fn load_snapshot(path: &Path) -> Result<StoreState, StoreError> {
    let bytes = fs::read(path).map_err(|e| StoreError::NotReady(format!("read snapshot: {e}")))?;
    let snapshot: StoreSnapshot = serde_json::from_slice(&bytes)
        .map_err(|e| StoreError::NotReady(format!("decode snapshot: {e}")))?;
    if snapshot.format_version != SNAPSHOT_FORMAT_VERSION {
        return Err(StoreError::NotReady(format!(
            "unsupported snapshot format {}",
            snapshot.format_version
        )));
    }
    Ok(StoreState::from(snapshot.state))
}
