use featherstore::*;
use std::collections::BTreeMap;

fn schema() -> FeatureSchema {
    FeatureSchema {
        entity: "user".into(),
        version: 1,
        features: vec![
            FeatureDef {
                name: "age_bucket".into(),
                dtype: FeatureDType::Int64,
                nullable: false,
            },
            FeatureDef {
                name: "score".into(),
                dtype: FeatureDType::Float64,
                nullable: false,
            },
            FeatureDef {
                name: "nickname".into(),
                dtype: FeatureDType::String,
                nullable: true,
            },
        ],
    }
}

fn valid_row() -> ApiFeatureRow {
    let mut values = BTreeMap::new();
    values.insert("age_bucket".into(), FeatureValue::Int64(3));
    values.insert("score".into(), FeatureValue::Float64(0.7));
    values.insert("nickname".into(), FeatureValue::Null);
    ApiFeatureRow {
        id: "42".into(),
        values,
        event_ts_ms: None,
    }
}

#[test]
fn valid_schema_and_rows_pass() {
    let req = IngestBatchRequest {
        schema: schema(),
        rows: vec![valid_row()],
        mode: IngestMode::Upsert,
    };
    validate_ingest_request(&req).unwrap();
}

#[test]
fn duplicate_feature_names_fail() {
    let mut s = schema();
    s.features.push(FeatureDef {
        name: "score".into(),
        dtype: FeatureDType::Float64,
        nullable: false,
    });
    let req = IngestBatchRequest {
        schema: s,
        rows: vec![valid_row()],
        mode: IngestMode::Upsert,
    };
    assert!(matches!(
        validate_ingest_request(&req),
        Err(StoreError::BadRequest(_))
    ));
}

#[test]
fn missing_required_value_fails() {
    let mut row = valid_row();
    row.values.remove("score");
    let req = IngestBatchRequest {
        schema: schema(),
        rows: vec![row],
        mode: IngestMode::Upsert,
    };
    assert!(validate_ingest_request(&req).is_err());
}

#[test]
fn unknown_row_value_name_fails() {
    let mut row = valid_row();
    row.values.insert("unknown".into(), FeatureValue::Int64(1));
    let req = IngestBatchRequest {
        schema: schema(),
        rows: vec![row],
        mode: IngestMode::Upsert,
    };
    assert!(validate_ingest_request(&req).is_err());
}

#[test]
fn null_non_nullable_value_fails() {
    let mut row = valid_row();
    row.values.insert("score".into(), FeatureValue::Null);
    let req = IngestBatchRequest {
        schema: schema(),
        rows: vec![row],
        mode: IngestMode::Upsert,
    };
    assert!(validate_ingest_request(&req).is_err());
}

#[test]
fn wrong_value_type_fails() {
    let mut row = valid_row();
    row.values
        .insert("score".into(), FeatureValue::String("bad".into()));
    let req = IngestBatchRequest {
        schema: schema(),
        rows: vec![row],
        mode: IngestMode::Upsert,
    };
    assert!(validate_ingest_request(&req).is_err());
}
