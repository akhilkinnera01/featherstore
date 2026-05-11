use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use featherstore::*;
use tower::ServiceExt;

#[tokio::test]
async fn health_ready_ingest_lookup_and_metrics_work() {
    let state = AppState::new(ServerConfig::default()).unwrap();
    let app = app(state);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/healthz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let ingest = r#"{"schema":{"entity":"user","version":1,"features":[{"name":"age_bucket","dtype":"int64","nullable":false}]},"rows":[{"id":"42","values":{"age_bucket":3},"event_ts_ms":null}],"mode":"upsert"}"#;
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/ingest/batch")
                .header("content-type", "application/json")
                .body(Body::from(ingest))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/features/user/42?features=age_bucket")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let response = app.clone().oneshot(Request::builder().method("POST").uri("/v1/features/lookup").header("content-type","application/json").body(Body::from(r#"{"requests":[{"entity":"user","id":"missing","features":null}],"include_missing":true}"#)).unwrap()).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/ingest/batch")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"schema":{"entity":"","version":1,"features":[]},"rows":[]}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/metrics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
