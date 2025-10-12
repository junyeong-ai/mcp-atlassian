use anyhow::{Context, Result};
use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    // Atlassian API Configuration
    pub atlassian_domain: String,
    pub atlassian_email: String,
    pub atlassian_api_token: String,

    // Performance
    pub max_connections: usize,
    pub request_timeout_ms: u64,

    // Project/Space Filtering
    pub jira_projects_filter: Vec<String>,
    pub confluence_spaces_filter: Vec<String>,

    // Jira Search Field Configuration
    pub jira_search_default_fields: Option<Vec<String>>,
    pub jira_search_custom_fields: Vec<String>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        // Load .env file if it exists
        dotenv::dotenv().ok();

        let domain = env::var("ATLASSIAN_DOMAIN")
            .context("ATLASSIAN_DOMAIN environment variable not set")?;

        tracing::debug!("Loaded ATLASSIAN_DOMAIN: {}", domain);

        // Parse Jira search field configuration
        let jira_search_default_fields: Option<Vec<String>> =
            env::var("JIRA_SEARCH_DEFAULT_FIELDS").ok().map(|s| {
                s.split(',')
                    .filter(|s| !s.trim().is_empty())
                    .map(|s| s.trim().to_string())
                    .collect()
            });

        let jira_search_custom_fields: Vec<String> = env::var("JIRA_SEARCH_CUSTOM_FIELDS")
            .unwrap_or_default()
            .split(',')
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.trim().to_string())
            .collect();

        if let Some(ref fields) = jira_search_default_fields {
            tracing::info!(
                "Using custom default fields from JIRA_SEARCH_DEFAULT_FIELDS: {} fields",
                fields.len()
            );
        }

        if !jira_search_custom_fields.is_empty() {
            tracing::info!(
                "Adding {} custom fields from JIRA_SEARCH_CUSTOM_FIELDS",
                jira_search_custom_fields.len()
            );
        }

        Ok(Self {
            atlassian_domain: domain,
            atlassian_email: env::var("ATLASSIAN_EMAIL")
                .context("ATLASSIAN_EMAIL environment variable not set")?,
            atlassian_api_token: env::var("ATLASSIAN_API_TOKEN")
                .context("ATLASSIAN_API_TOKEN environment variable not set")?,

            max_connections: env::var("MAX_CONNECTIONS")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .context("Invalid MAX_CONNECTIONS")?,
            request_timeout_ms: env::var("REQUEST_TIMEOUT_MS")
                .unwrap_or_else(|_| "30000".to_string())
                .parse()
                .context("Invalid REQUEST_TIMEOUT_MS")?,

            jira_projects_filter: env::var("JIRA_PROJECTS_FILTER")
                .unwrap_or_default()
                .split(',')
                .filter(|s| !s.is_empty())
                .map(|s| s.trim().to_string())
                .collect(),
            confluence_spaces_filter: env::var("CONFLUENCE_SPACES_FILTER")
                .unwrap_or_default()
                .split(',')
                .filter(|s| !s.is_empty())
                .map(|s| s.trim().to_string())
                .collect(),

            jira_search_default_fields,
            jira_search_custom_fields,
        })
    }

    pub fn validate(&self) -> Result<()> {
        if self.atlassian_domain.is_empty() {
            anyhow::bail!("Atlassian domain cannot be empty");
        }

        // Check if it's a valid Atlassian domain
        let domain = if self.atlassian_domain.starts_with("https://") {
            &self.atlassian_domain[8..]
        } else if self.atlassian_domain.starts_with("http://") {
            &self.atlassian_domain[7..]
        } else {
            &self.atlassian_domain
        };

        if !domain.contains(".atlassian.net") {
            anyhow::bail!("Invalid Atlassian domain format");
        }

        if self.atlassian_email.is_empty() || !self.atlassian_email.contains('@') {
            anyhow::bail!("Invalid Atlassian email");
        }

        if self.atlassian_api_token.is_empty() {
            anyhow::bail!("API token cannot be empty");
        }

        if self.max_connections == 0 || self.max_connections > 1000 {
            anyhow::bail!("Max connections must be between 1 and 1000");
        }

        if self.request_timeout_ms < 100 || self.request_timeout_ms > 60000 {
            anyhow::bail!("Request timeout must be between 100ms and 60000ms");
        }

        Ok(())
    }

    pub fn get_atlassian_base_url(&self) -> String {
        // If domain already starts with https://, don't add it again
        if self.atlassian_domain.starts_with("https://") {
            self.atlassian_domain.clone()
        } else if self.atlassian_domain.starts_with("http://") {
            // Replace http with https
            self.atlassian_domain.replace("http://", "https://")
        } else {
            format!("https://{}", self.atlassian_domain)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // T009: Valid configuration tests
    #[test]
    fn test_config_validation() {
        let config = Config {
            atlassian_domain: "test.atlassian.net".to_string(),
            atlassian_email: "test@example.com".to_string(),
            atlassian_api_token: "token123".to_string(),
            max_connections: 100,
            request_timeout_ms: 30000,
            jira_projects_filter: vec![],
            confluence_spaces_filter: vec![],
            jira_search_default_fields: None,
            jira_search_custom_fields: vec![],
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_domain_normalization_adds_https() {
        let config = Config {
            atlassian_domain: "test.atlassian.net".to_string(),
            atlassian_email: "test@example.com".to_string(),
            atlassian_api_token: "token".to_string(),
            max_connections: 10,
            request_timeout_ms: 30000,
            jira_projects_filter: vec![],
            confluence_spaces_filter: vec![],
            jira_search_default_fields: None,
            jira_search_custom_fields: vec![],
        };

        let url = config.get_atlassian_base_url();
        assert!(url.starts_with("https://"));
        assert_eq!(url, "https://test.atlassian.net");
    }

    #[test]
    fn test_domain_normalization_converts_http_to_https() {
        let config = Config {
            atlassian_domain: "http://test.atlassian.net".to_string(),
            atlassian_email: "test@example.com".to_string(),
            atlassian_api_token: "token".to_string(),
            max_connections: 10,
            request_timeout_ms: 30000,
            jira_projects_filter: vec![],
            confluence_spaces_filter: vec![],
            jira_search_default_fields: None,
            jira_search_custom_fields: vec![],
        };

        let url = config.get_atlassian_base_url();
        assert!(url.starts_with("https://"));
        assert!(!url.contains("http://"));
    }

    // T010: Invalid configuration tests
    #[test]
    fn test_invalid_domain() {
        let config = Config {
            atlassian_domain: "invalid-domain".to_string(),
            atlassian_email: "test@example.com".to_string(),
            atlassian_api_token: "token123".to_string(),
            max_connections: 100,
            request_timeout_ms: 30000,
            jira_projects_filter: vec![],
            confluence_spaces_filter: vec![],
            jira_search_default_fields: None,
            jira_search_custom_fields: vec![],
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_email_missing_at_symbol() {
        let config = Config {
            atlassian_domain: "test.atlassian.net".to_string(),
            atlassian_email: "invalid-email".to_string(),
            atlassian_api_token: "token".to_string(),
            max_connections: 10,
            request_timeout_ms: 30000,
            jira_projects_filter: vec![],
            confluence_spaces_filter: vec![],
            jira_search_default_fields: None,
            jira_search_custom_fields: vec![],
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_empty_api_token_fails() {
        let config = Config {
            atlassian_domain: "test.atlassian.net".to_string(),
            atlassian_email: "test@example.com".to_string(),
            atlassian_api_token: "".to_string(),
            max_connections: 10,
            request_timeout_ms: 30000,
            jira_projects_filter: vec![],
            confluence_spaces_filter: vec![],
            jira_search_default_fields: None,
            jira_search_custom_fields: vec![],
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_max_connections_zero() {
        let config = Config {
            atlassian_domain: "test.atlassian.net".to_string(),
            atlassian_email: "test@example.com".to_string(),
            atlassian_api_token: "token".to_string(),
            max_connections: 0,
            request_timeout_ms: 30000,
            jira_projects_filter: vec![],
            confluence_spaces_filter: vec![],
            jira_search_default_fields: None,
            jira_search_custom_fields: vec![],
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_timeout_too_low() {
        let config = Config {
            atlassian_domain: "test.atlassian.net".to_string(),
            atlassian_email: "test@example.com".to_string(),
            atlassian_api_token: "token".to_string(),
            max_connections: 10,
            request_timeout_ms: 50,
            jira_projects_filter: vec![],
            confluence_spaces_filter: vec![],
            jira_search_default_fields: None,
            jira_search_custom_fields: vec![],
        };

        assert!(config.validate().is_err());
    }

    // T024: Additional configuration tests for coverage
    #[test]
    fn test_url_normalization_preserves_https() {
        let config = Config {
            atlassian_domain: "https://test.atlassian.net".to_string(),
            atlassian_email: "test@example.com".to_string(),
            atlassian_api_token: "token".to_string(),
            max_connections: 10,
            request_timeout_ms: 30000,
            jira_projects_filter: vec![],
            confluence_spaces_filter: vec![],
            jira_search_default_fields: None,
            jira_search_custom_fields: vec![],
        };

        let url = config.get_atlassian_base_url();
        assert_eq!(url, "https://test.atlassian.net");
        assert_eq!(url.matches("https://").count(), 1); // Exactly one https://
    }

    #[test]
    fn test_max_connections_upper_bound() {
        let config = Config {
            atlassian_domain: "test.atlassian.net".to_string(),
            atlassian_email: "test@example.com".to_string(),
            atlassian_api_token: "token".to_string(),
            max_connections: 1001, // Above max
            request_timeout_ms: 30000,
            jira_projects_filter: vec![],
            confluence_spaces_filter: vec![],
            jira_search_default_fields: None,
            jira_search_custom_fields: vec![],
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_request_timeout_upper_bound() {
        let config = Config {
            atlassian_domain: "test.atlassian.net".to_string(),
            atlassian_email: "test@example.com".to_string(),
            atlassian_api_token: "token".to_string(),
            max_connections: 10,
            request_timeout_ms: 60001, // Above max
            jira_projects_filter: vec![],
            confluence_spaces_filter: vec![],
            jira_search_default_fields: None,
            jira_search_custom_fields: vec![],
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_project_filter_with_values() {
        let config = Config {
            atlassian_domain: "test.atlassian.net".to_string(),
            atlassian_email: "test@example.com".to_string(),
            atlassian_api_token: "token".to_string(),
            max_connections: 10,
            request_timeout_ms: 30000,
            jira_projects_filter: vec!["PROJ1".to_string(), "PROJ2".to_string()],
            confluence_spaces_filter: vec![],
            jira_search_default_fields: None,
            jira_search_custom_fields: vec![],
        };

        assert!(config.validate().is_ok());
        assert_eq!(config.jira_projects_filter.len(), 2);
        assert_eq!(config.jira_projects_filter[0], "PROJ1");
    }

    #[test]
    fn test_space_filter_with_values() {
        let config = Config {
            atlassian_domain: "test.atlassian.net".to_string(),
            atlassian_email: "test@example.com".to_string(),
            atlassian_api_token: "token".to_string(),
            max_connections: 10,
            request_timeout_ms: 30000,
            jira_projects_filter: vec![],
            confluence_spaces_filter: vec!["SPACE1".to_string(), "SPACE2".to_string()],
            jira_search_default_fields: None,
            jira_search_custom_fields: vec![],
        };

        assert!(config.validate().is_ok());
        assert_eq!(config.confluence_spaces_filter.len(), 2);
        assert_eq!(config.confluence_spaces_filter[0], "SPACE1");
    }

    #[test]
    fn test_jira_search_custom_fields_configuration() {
        let config = Config {
            atlassian_domain: "test.atlassian.net".to_string(),
            atlassian_email: "test@example.com".to_string(),
            atlassian_api_token: "token".to_string(),
            max_connections: 10,
            request_timeout_ms: 30000,
            jira_projects_filter: vec![],
            confluence_spaces_filter: vec![],
            jira_search_default_fields: None,
            jira_search_custom_fields: vec![
                "customfield_10015".to_string(),
                "customfield_10016".to_string(),
            ],
        };

        assert!(config.validate().is_ok());
        assert_eq!(config.jira_search_custom_fields.len(), 2);
        assert!(config
            .jira_search_custom_fields
            .contains(&"customfield_10015".to_string()));
    }

    #[test]
    fn test_jira_search_default_fields_override() {
        let config = Config {
            atlassian_domain: "test.atlassian.net".to_string(),
            atlassian_email: "test@example.com".to_string(),
            atlassian_api_token: "token".to_string(),
            max_connections: 10,
            request_timeout_ms: 30000,
            jira_projects_filter: vec![],
            confluence_spaces_filter: vec![],
            jira_search_default_fields: Some(vec![
                "key".to_string(),
                "summary".to_string(),
                "status".to_string(),
            ]),
            jira_search_custom_fields: vec![],
        };

        assert!(config.validate().is_ok());
        assert!(config.jira_search_default_fields.is_some());
        assert_eq!(config.jira_search_default_fields.as_ref().unwrap().len(), 3);
    }

    #[test]
    fn test_domain_with_https_validates() {
        let config = Config {
            atlassian_domain: "https://test.atlassian.net".to_string(),
            atlassian_email: "test@example.com".to_string(),
            atlassian_api_token: "token".to_string(),
            max_connections: 10,
            request_timeout_ms: 30000,
            jira_projects_filter: vec![],
            confluence_spaces_filter: vec![],
            jira_search_default_fields: None,
            jira_search_custom_fields: vec![],
        };

        assert!(config.validate().is_ok());
    }
}
