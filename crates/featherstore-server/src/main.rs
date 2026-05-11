use anyhow::Context;
use featherstore::{app, AppState, LogFormat, ServerConfig};
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = ServerConfig::from_env().map_err(anyhow::Error::msg)?;
    init_tracing(&config);
    let state = AppState::new(config.clone())?;
    let listener = tokio::net::TcpListener::bind(config.bind_addr)
        .await
        .context("bind listener")?;
    tracing::info!(bind_addr = %config.bind_addr, "starting featherstore server");
    axum::serve(listener, app(state))
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("serve http")?;
    Ok(())
}

fn init_tracing(config: &ServerConfig) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    match config.log_format {
        LogFormat::Json => fmt().json().with_env_filter(filter).init(),
        LogFormat::Compact => fmt().compact().with_env_filter(filter).init(),
    }
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
    tracing::info!("shutdown signal received");
}
