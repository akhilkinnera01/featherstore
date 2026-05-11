use featherstore::*;

#[test]
fn architecture_ingest_example_deserializes() {
    let json = r#"{
      "schema": {"entity":"user","version":1,"features":[{"name":"age_bucket","dtype":"int64","nullable":false},{"name":"country_id","dtype":"int64","nullable":false},{"name":"u_emb_0","dtype":"float64","nullable":false}]},
      "rows": [{"id":"42","values":{"age_bucket":3,"country_id":840,"u_emb_0":0.123},"event_ts_ms":1778520000000}],
      "mode": "upsert"
    }"#;
    let req: IngestBatchRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.schema.entity, "user");
    assert_eq!(req.rows.len(), 1);
}

#[test]
fn error_response_shape_is_stable() {
    let err = StoreError::BadRequest("bad".into());
    assert_eq!(err.kind(), "bad_request");
    assert_eq!(err.status(), http::StatusCode::BAD_REQUEST);
}
