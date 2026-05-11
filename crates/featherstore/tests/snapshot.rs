use featherstore::*;
use std::collections::BTreeMap;

#[test]
fn snapshot_round_trip_preserves_rows_and_schemas() {
    let temp = std::env::temp_dir().join(format!("featherstore-{}.bin", std::process::id()));
    let store = InMemoryFeatureStore::new();
    let schema = FeatureSchema {
        entity: "user".into(),
        version: 1,
        features: vec![FeatureDef {
            name: "age".into(),
            dtype: FeatureDType::Int64,
            nullable: false,
        }],
    };
    let mut values = BTreeMap::new();
    values.insert("age".into(), FeatureValue::Int64(1));
    store
        .upsert_batch(IngestBatchRequest {
            schema,
            rows: vec![ApiFeatureRow {
                id: "1".into(),
                values,
                event_ts_ms: None,
            }],
            mode: IngestMode::Upsert,
        })
        .unwrap();
    snapshot::save_snapshot(&temp, &store.export_state()).unwrap();
    let restored = InMemoryFeatureStore::from_state(snapshot::load_snapshot(&temp).unwrap());
    assert!(restored.get_one("user", "1", None).unwrap().found);
    let _ = std::fs::remove_file(temp);
}

#[test]
fn corrupt_snapshot_returns_error() {
    let temp =
        std::env::temp_dir().join(format!("featherstore-corrupt-{}.bin", std::process::id()));
    std::fs::write(&temp, b"not a snapshot").unwrap();
    assert!(snapshot::load_snapshot(&temp).is_err());
    let _ = std::fs::remove_file(temp);
}
