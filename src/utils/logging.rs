use std::env;
use tracing::info;
use tracing_subscriber::{
    EnvFilter,
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

pub fn init_logging() {
    let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "warn".to_string());

    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level));

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
        "MCP Atlassian server starting (stdio mode)"
    );
}

pub fn log_shutdown() {
    info!("MCP Atlassian server shutting down");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    fn create_test_config() -> Config {
        Config {
            atlassian_domain: "test.atlassian.net".to_string(),
            atlassian_email: "test@example.com".to_string(),
            atlassian_api_token: "token123".to_string(),
            request_timeout_ms: 30000,
            jira_projects_filter: vec![],
            confluence_spaces_filter: vec![],
            jira_search_default_fields: None,
            jira_search_custom_fields: vec![],
        }
    }

    // T019: Logging tests

    #[test]
    fn test_log_startup_format() {
        // Test that log_startup doesn't panic
        let config = create_test_config();
        log_startup(&config);
        // If we get here, the function executed successfully
    }

    #[test]
    fn test_log_shutdown_format() {
        // Test that log_shutdown doesn't panic
        log_shutdown();
        // If we get here, the function executed successfully
    }

    #[test]
    fn test_log_macros_compile() {
        // Test that log macros compile correctly
        // These are macro tests - we just verify they expand without errors

        // log_request macro
        let request_id = "req-123";
        let method = "tools/call";
        let params = serde_json::json!({"tool": "jira_search"});
        log_request!(request_id, method, params);

        // log_response macro
        log_response!(request_id, 150, 1000);

        // log_error macro
        let error = "Test error";
        log_error!(request_id, error);

        // If we get here, all macros expanded successfully
    }
}
