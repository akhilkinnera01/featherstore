use featherstore::*;
use std::collections::BTreeMap;

fn ingest_request() -> IngestBatchRequest {
    let schema = FeatureSchema {
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
        ],
    };
    let mut values = BTreeMap::new();
    values.insert("age_bucket".into(), FeatureValue::Int64(4));
    values.insert("score".into(), FeatureValue::Float64(0.9));
    IngestBatchRequest {
        schema,
        rows: vec![ApiFeatureRow {
            id: "42".into(),
            values,
            event_ts_ms: None,
        }],
        mode: IngestMode::Upsert,
    }
}

#[test]
fn upsert_then_lookup_all_features() {
    let store = InMemoryFeatureStore::new();
    store.upsert_batch(ingest_request()).unwrap();
    let row = store.get_one("user", "42", None).unwrap();
    assert!(row.found);
    assert_eq!(row.values.len(), 2);
}

#[test]
fn upsert_then_lookup_subset() {
    let store = InMemoryFeatureStore::new();
    store.upsert_batch(ingest_request()).unwrap();
    let row = store
        .get_one("user", "42", Some(&["score".to_string()]))
        .unwrap();
    assert_eq!(row.values.len(), 1);
    assert!(row.values.contains_key("score"));
}

#[test]
fn missing_key_returns_found_false() {
    let store = InMemoryFeatureStore::new();
    store.upsert_batch(ingest_request()).unwrap();
    let row = store.get_one("user", "missing", None).unwrap();
    assert!(!row.found);
    assert!(row.values.is_empty());
}

#[test]
fn unknown_feature_subset_returns_bad_request() {
    let store = InMemoryFeatureStore::new();
    store.upsert_batch(ingest_request()).unwrap();
    assert!(matches!(
        store.get_one("user", "42", Some(&["nope".to_string()])),
        Err(StoreError::BadRequest(_))
    ));
}

#[test]
fn stats_are_correct() {
    let store = InMemoryFeatureStore::new();
    store.upsert_batch(ingest_request()).unwrap();
    let stats = store.stats();
    assert_eq!(stats.schemas, 1);
    assert_eq!(stats.rows, 1);
    assert_eq!(stats.rows_by_entity.get("user"), Some(&1));
}
