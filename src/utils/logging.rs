use tracing::info;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};
use std::env;

pub fn init_logging() {
    let log_level = env::var("LOG_LEVEL")
        .unwrap_or_else(|_| "warn".to_string());

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(log_level));

    // IMPORTANT: Always write logs to stderr, never stdout!
    // stdout is reserved for MCP protocol messages only
    let fmt_layer = fmt::layer()
        .with_target(false)
        .with_level(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_span_events(FmtSpan::CLOSE)
        .with_ansi(false)
        .with_writer(std::io::stderr);

    let json_logs = env::var("JSON_LOGS")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap_or(false);

    if json_logs {
        let fmt_layer = fmt_layer.json();
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .init();
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .init();
    }

    info!("Logging initialized");
}

#[macro_export]
macro_rules! log_request {
    ($request_id:expr, $method:expr, $params:expr) => {
        tracing::info!(
            request_id = %$request_id,
            method = %$method,
            params = ?$params,
            "Processing MCP request"
        );
    };
}

#[macro_export]
macro_rules! log_response {
    ($request_id:expr, $latency_ms:expr, $token_count:expr) => {
        tracing::info!(
            request_id = %$request_id,
            latency_ms = $latency_ms,
            token_count = $token_count,
            "MCP response sent"
        );
    };
}

#[macro_export]
macro_rules! log_error {
    ($request_id:expr, $error:expr) => {
        tracing::error!(
            request_id = %$request_id,
            error = %$error,
            "Request failed"
        );
    };
}

pub fn log_startup(config: &crate::config::Config) {
    info!(
        atlassian_domain = %config.atlassian_domain,
        max_connections = config.max_connections,
        "MCP Atlassian server starting (stdio mode)"
    );
}

pub fn log_shutdown() {
    info!("MCP Atlassian server shutting down");
}