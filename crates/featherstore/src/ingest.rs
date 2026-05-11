use crate::{FeatureDType, FeatureValue, IngestBatchRequest, StoreError};
use std::collections::HashSet;

pub fn validate_ingest_request(req: &IngestBatchRequest) -> Result<(), StoreError> {
    let entity = req.schema.entity.trim();
    if entity.is_empty() {
        return Err(StoreError::BadRequest(
            "schema entity must be non-empty".into(),
        ));
    }
    let mut names = HashSet::new();
    for feature in &req.schema.features {
        let name = feature.name.trim();
        if name.is_empty() {
            return Err(StoreError::BadRequest(
                "feature names must be non-empty".into(),
            ));
        }
        if !names.insert(feature.name.as_str()) {
            return Err(StoreError::BadRequest(format!(
                "duplicate feature name '{}'",
                feature.name
            )));
        }
    }
    for row in &req.rows {
        if row.id.trim().is_empty() {
            return Err(StoreError::BadRequest("row id must be non-empty".into()));
        }
        for name in row.values.keys() {
            if !names.contains(name.as_str()) {
                return Err(StoreError::BadRequest(format!(
                    "row '{}' contains unknown feature '{}'",
                    row.id, name
                )));
            }
        }
        for def in &req.schema.features {
            match row.values.get(&def.name) {
                None if !def.nullable => {
                    return Err(StoreError::BadRequest(format!(
                        "row '{}' missing required feature '{}'",
                        row.id, def.name
                    )))
                }
                None => {}
                Some(FeatureValue::Null) if !def.nullable => {
                    return Err(StoreError::BadRequest(format!(
                        "feature '{}' is not nullable",
                        def.name
                    )))
                }
                Some(value) => validate_value_type(&def.name, &def.dtype, value)?,
            }
        }
    }
    Ok(())
}

fn validate_value_type(
    name: &str,
    dtype: &FeatureDType,
    value: &FeatureValue,
) -> Result<(), StoreError> {
    let ok = matches!(
        (dtype, value),
        (_, FeatureValue::Null)
            | (FeatureDType::Int64, FeatureValue::Int64(_))
            | (FeatureDType::Float64, FeatureValue::Float64(_))
            | (FeatureDType::Bool, FeatureValue::Bool(_))
            | (FeatureDType::String, FeatureValue::String(_))
    );
    if ok {
        Ok(())
    } else {
        Err(StoreError::BadRequest(format!(
            "feature '{}' has wrong value type",
            name
        )))
    }
}
