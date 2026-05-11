use crate::model::{ApiFeatureRow, EntityId, EntityType, FeatureName, FeatureSchema, FeatureValue};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IngestBatchRequest {
    pub schema: FeatureSchema,
    pub rows: Vec<ApiFeatureRow>,
    #[serde(default = "default_ingest_mode")]
    pub mode: IngestMode,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IngestMode {
    Upsert,
    ReplaceEntity,
}

pub fn default_ingest_mode() -> IngestMode {
    IngestMode::Upsert
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct IngestSummary {
    pub entity: EntityType,
    pub accepted_rows: usize,
    pub rejected_rows: usize,
    pub total_rows: usize,
    pub schema_version: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LookupRequest {
    pub requests: Vec<EntityLookupRequest>,
    #[serde(default = "default_include_missing")]
    pub include_missing: bool,
}

pub fn default_include_missing() -> bool {
    true
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntityLookupRequest {
    pub entity: EntityType,
    pub id: EntityId,
    pub features: Option<Vec<FeatureName>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LookupResponse {
    pub rows: Vec<LookupRow>,
    pub elapsed_us: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LookupRow {
    pub entity: EntityType,
    pub id: EntityId,
    pub found: bool,
    pub values: BTreeMap<FeatureName, FeatureValue>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_include_missing_defaults_true() {
        let json = r#"{"requests":[{"entity":"user","id":"42","features":null}]}"#;
        let req: LookupRequest = serde_json::from_str(json).unwrap();
        assert!(req.include_missing);
    }

    #[test]
    fn invalid_ingest_mode_fails_deserialization() {
        let json =
            r#"{"schema":{"entity":"user","version":1,"features":[]},"rows":[],"mode":"append"}"#;
        assert!(serde_json::from_str::<IngestBatchRequest>(json).is_err());
    }
}
