use prometheus::{
    Encoder, HistogramOpts, HistogramVec, IntCounterVec, IntGauge, IntGaugeVec, Opts, Registry,
    TextEncoder,
};

#[derive(Clone)]
pub struct Metrics {
    registry: Registry,
    pub http_requests: IntCounterVec,
    pub http_duration: HistogramVec,
    pub lookup_requests: IntCounterVec,
    pub lookup_rows: IntCounterVec,
    pub ingest_rows: IntCounterVec,
    pub store_rows: IntGaugeVec,
    pub store_schemas: IntGauge,
    pub errors: IntCounterVec,
}

impl Metrics {
    pub fn new() -> Result<Self, prometheus::Error> {
        let registry = Registry::new();
        let http_requests = IntCounterVec::new(
            Opts::new(
                "featherstore_http_requests_total",
                "HTTP requests by route/status",
            ),
            &["method", "path", "status"],
        )?;
        let http_duration = HistogramVec::new(
            HistogramOpts::new(
                "featherstore_http_request_duration_seconds",
                "HTTP request latency",
            )
            .buckets(vec![
                0.00025, 0.0005, 0.001, 0.0025, 0.005, 0.0075, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5,
                1.0,
            ]),
            &["method", "path", "status"],
        )?;
        let lookup_requests = IntCounterVec::new(
            Opts::new(
                "featherstore_lookup_requests_total",
                "Lookup requests by entity/status",
            ),
            &["entity", "status"],
        )?;
        let lookup_rows = IntCounterVec::new(
            Opts::new(
                "featherstore_lookup_rows_total",
                "Lookup rows by entity/found",
            ),
            &["entity", "found"],
        )?;
        let ingest_rows = IntCounterVec::new(
            Opts::new(
                "featherstore_ingest_rows_total",
                "Ingest rows by entity/status",
            ),
            &["entity", "status"],
        )?;
        let store_rows = IntGaugeVec::new(
            Opts::new(
                "featherstore_store_rows",
                "Rows currently in store by entity",
            ),
            &["entity"],
        )?;
        let store_schemas =
            IntGauge::new("featherstore_store_schemas", "Schemas currently loaded")?;
        let errors = IntCounterVec::new(
            Opts::new("featherstore_errors_total", "Errors by kind"),
            &["kind"],
        )?;
        registry.register(Box::new(http_requests.clone()))?;
        registry.register(Box::new(http_duration.clone()))?;
        registry.register(Box::new(lookup_requests.clone()))?;
        registry.register(Box::new(lookup_rows.clone()))?;
        registry.register(Box::new(ingest_rows.clone()))?;
        registry.register(Box::new(store_rows.clone()))?;
        registry.register(Box::new(store_schemas.clone()))?;
        registry.register(Box::new(errors.clone()))?;
        Ok(Self {
            registry,
            http_requests,
            http_duration,
            lookup_requests,
            lookup_rows,
            ingest_rows,
            store_rows,
            store_schemas,
            errors,
        })
    }

    pub fn render(&self) -> Result<String, prometheus::Error> {
        let mut buf = Vec::new();
        TextEncoder::new().encode(&self.registry.gather(), &mut buf)?;
        Ok(String::from_utf8_lossy(&buf).into_owned())
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new().expect("metrics registry can be initialized")
    }
}
