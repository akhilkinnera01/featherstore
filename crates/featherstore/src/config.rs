use std::{env, net::SocketAddr, path::PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LogFormat {
    Compact,
    Json,
}

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub bind_addr: SocketAddr,
    pub log_format: LogFormat,
    pub max_request_bytes: usize,
    pub snapshot_path: Option<PathBuf>,
    pub persist_snapshot: bool,
    pub load_snapshot_on_start: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_addr: "0.0.0.0:8080".parse().expect("valid default bind address"),
            log_format: LogFormat::Compact,
            max_request_bytes: 16 * 1024 * 1024,
            snapshot_path: None,
            persist_snapshot: false,
            load_snapshot_on_start: false,
        }
    }
}

impl ServerConfig {
    pub fn from_env() -> Result<Self, String> {
        let mut cfg = Self::default();
        if let Ok(v) = env::var("FEATHERSTORE_BIND_ADDR") {
            cfg.bind_addr = v
                .parse()
                .map_err(|e| format!("invalid FEATHERSTORE_BIND_ADDR: {e}"))?;
        }
        if let Ok(v) = env::var("FEATHERSTORE_LOG_FORMAT") {
            cfg.log_format = match v.as_str() {
                "json" => LogFormat::Json,
                "compact" => LogFormat::Compact,
                other => return Err(format!("invalid FEATHERSTORE_LOG_FORMAT '{other}'")),
            };
        }
        if let Ok(v) = env::var("FEATHERSTORE_MAX_REQUEST_BYTES") {
            cfg.max_request_bytes = v
                .parse()
                .map_err(|e| format!("invalid FEATHERSTORE_MAX_REQUEST_BYTES: {e}"))?;
        }
        if let Ok(v) = env::var("FEATHERSTORE_SNAPSHOT_PATH") {
            if !v.is_empty() {
                cfg.snapshot_path = Some(PathBuf::from(v));
            }
        }
        cfg.persist_snapshot = env_bool("FEATHERSTORE_PERSIST_SNAPSHOT", false);
        cfg.load_snapshot_on_start = env_bool("FEATHERSTORE_LOAD_SNAPSHOT_ON_START", false);
        Ok(cfg)
    }
}

fn env_bool(name: &str, default: bool) -> bool {
    env::var(name)
        .ok()
        .and_then(|v| match v.to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => Some(true),
            "0" | "false" | "no" | "off" => Some(false),
            _ => None,
        })
        .unwrap_or(default)
}
