use crate::config::Config;
use reqwest::Client;
use std::time::Duration;

pub fn create_atlassian_client(config: &Config) -> Client {
    Client::builder()
        .timeout(Duration::from_millis(config.request_timeout_ms))
        .build()
        .expect("Failed to create HTTP client")
}

pub fn create_auth_header(config: &Config) -> String {
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    let credentials = format!("{}:{}", config.atlassian_email, config.atlassian_api_token);
    format!("Basic {}", STANDARD.encode(credentials))
}
