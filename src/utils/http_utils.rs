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
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    let credentials = format!("{}:{}", config.atlassian_email, config.atlassian_api_token);
    format!("Basic {}", STANDARD.encode(credentials))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config(
        email: &str,
        token: &str,
        timeout_ms: u64,
        max_connections: usize,
    ) -> Config {
        Config {
            atlassian_domain: "test.atlassian.net".to_string(),
            atlassian_email: email.to_string(),
            atlassian_api_token: token.to_string(),
            max_connections,
            request_timeout_ms: timeout_ms,
            jira_projects_filter: vec![],
            confluence_spaces_filter: vec![],
            jira_search_default_fields: None,
            jira_search_custom_fields: vec![],
        }
    }

    // T019: HTTP Utils tests

    #[test]
    fn test_create_atlassian_client_success() {
        let config = create_test_config("test@example.com", "token123", 30000, 100);
        let client = create_atlassian_client(&config);

        // Client should be created successfully
        // We can't directly test timeout value, but we can verify client is created
        assert!(format!("{:?}", client).contains("Client"));
    }

    #[test]
    fn test_create_atlassian_client_with_custom_timeout() {
        let config = create_test_config("test@example.com", "token123", 5000, 100);
        let client = create_atlassian_client(&config);

        // Client should respect custom timeout configuration
        assert!(format!("{:?}", client).contains("Client"));
    }

    #[test]
    fn test_create_auth_header_format() {
        let config = create_test_config("user@example.com", "secret123", 30000, 100);
        let auth_header = create_auth_header(&config);

        // Should start with "Basic "
        assert!(auth_header.starts_with("Basic "));

        // Should contain base64-encoded credentials
        let base64_part = &auth_header[6..]; // Skip "Basic "
        assert!(!base64_part.is_empty());
    }

    #[test]
    fn test_create_auth_header_base64_encoding() {
        let config = create_test_config("test@example.com", "mytoken", 30000, 100);
        let auth_header = create_auth_header(&config);

        // Decode and verify the credentials
        use base64::{Engine as _, engine::general_purpose::STANDARD};
        let base64_part = &auth_header[6..];
        let decoded = STANDARD.decode(base64_part).unwrap();
        let credentials = String::from_utf8(decoded).unwrap();

        assert_eq!(credentials, "test@example.com:mytoken");
    }

    #[test]
    fn test_create_auth_header_with_special_characters() {
        // Test with special characters in email and token
        let config = create_test_config("user+test@example.com", "token!@#$%^&*()", 30000, 100);
        let auth_header = create_auth_header(&config);

        // Should properly encode special characters
        assert!(auth_header.starts_with("Basic "));

        use base64::{Engine as _, engine::general_purpose::STANDARD};
        let base64_part = &auth_header[6..];
        let decoded = STANDARD.decode(base64_part).unwrap();
        let credentials = String::from_utf8(decoded).unwrap();

        assert_eq!(credentials, "user+test@example.com:token!@#$%^&*()");
    }

    #[test]
    fn test_create_auth_header_deterministic() {
        // Same config should produce same header
        let config = create_test_config("test@example.com", "token123", 30000, 100);
        let header1 = create_auth_header(&config);
        let header2 = create_auth_header(&config);

        assert_eq!(header1, header2);
    }

    #[test]
    fn test_create_auth_header_different_configs() {
        // Different configs should produce different headers
        let config1 = create_test_config("user1@example.com", "token1", 30000, 100);
        let config2 = create_test_config("user2@example.com", "token2", 30000, 100);

        let header1 = create_auth_header(&config1);
        let header2 = create_auth_header(&config2);

        assert_ne!(header1, header2);
    }
}
