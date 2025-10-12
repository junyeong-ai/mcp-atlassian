
/// Configuration for Confluence API v2 parameters
#[derive(Debug, Clone)]
pub struct FieldConfiguration {
    pub body_format: Option<String>,
    pub include_version: bool,
    pub include_labels: bool,
    pub include_properties: bool,
    pub include_operations: bool,
    pub custom_includes: Vec<String>,
    pub include_all: bool,
}

impl FieldConfiguration {
    /// Create a new field configuration with essential parameters for v2 API
    pub fn from_env() -> Self {
        let custom_includes_env = std::env::var("CONFLUENCE_CUSTOM_INCLUDES").unwrap_or_default();
        let custom_includes: Vec<String> = if custom_includes_env.is_empty() {
            vec![]
        } else {
            custom_includes_env
                .split(',')
                .filter(|s| !s.is_empty())
                .map(|s| s.trim().to_string())
                .collect()
        };

        if !custom_includes.is_empty() {
            tracing::info!("Loaded {} custom include parameters: {:?}", custom_includes.len(), custom_includes);
        }

        Self {
            body_format: Some("storage".to_string()),
            include_version: true,
            include_labels: false,
            include_properties: false,
            include_operations: false,
            custom_includes,
            include_all: false,
        }
    }

    /// Create a configuration that includes all fields
    pub fn all_fields() -> Self {
        Self {
            body_format: Some("storage".to_string()),
            include_version: true,
            include_labels: true,
            include_properties: true,
            include_operations: true,
            custom_includes: vec![],
            include_all: true,
        }
    }

    /// Override to include additional parameters for a specific request
    pub fn with_additional_includes(&self, additional: Vec<String>) -> Self {
        let mut config = self.clone();
        for param in additional {
            if !config.custom_includes.contains(&param) {
                config.custom_includes.push(param);
            }
        }
        config
    }

    /// Get query parameters as a vector of tuples for v2 API
    pub fn to_query_params(&self) -> Vec<(String, String)> {
        let mut params = Vec::new();

        // Add body-format parameter
        if let Some(ref format) = self.body_format {
            params.push(("body-format".to_string(), format.clone()));
        }

        // Add include-* boolean parameters
        if self.include_version {
            params.push(("include-version".to_string(), "true".to_string()));
        }
        if self.include_labels || self.include_all {
            params.push(("include-labels".to_string(), "true".to_string()));
        }
        if self.include_properties || self.include_all {
            params.push(("include-properties".to_string(), "true".to_string()));
        }
        if self.include_operations || self.include_all {
            params.push(("include-operations".to_string(), "true".to_string()));
        }

        // Add custom include parameters
        for param in &self.custom_includes {
            params.push((format!("include-{}", param), "true".to_string()));
        }

        params
    }
}

/// Builds query parameters for Confluence API v2 requests
#[derive(Debug, Clone)]
pub struct FieldSelector {
    config: FieldConfiguration,
}

impl FieldSelector {
    /// Create a new field selector from configuration
    pub fn from_config(config: &FieldConfiguration) -> Self {
        Self {
            config: config.clone(),
        }
    }

    /// Create a selector for all fields (no filtering)
    pub fn all_fields() -> Self {
        Self {
            config: FieldConfiguration::all_fields(),
        }
    }

    /// Get query parameters for v2 API
    pub fn to_query_params(&self) -> Vec<(String, String)> {
        self.config.to_query_params()
    }
}

/// Helper function to apply field filtering for v2 API
pub fn apply_v2_filtering(
    include_all_fields: Option<bool>,
    additional_includes: Option<Vec<String>>,
) -> Vec<(String, String)> {
    if include_all_fields.unwrap_or(false) {
        tracing::debug!("Field filtering disabled: include_all_fields=true");
        let selector = FieldSelector::all_fields();
        return selector.to_query_params();
    }

    let mut config = FieldConfiguration::from_env();
    tracing::debug!("Loaded {} custom include parameters from environment", config.custom_includes.len());

    if let Some(additional) = additional_includes {
        tracing::debug!("Adding {} additional include parameters", additional.len());
        config = config.with_additional_includes(additional);
    }

    let selector = FieldSelector::from_config(&config);
    let params = selector.to_query_params();

    tracing::debug!("Applying v2 filtering with {} parameters", params.len());

    params
}

/// Legacy helper for v1 search endpoint (still uses expand parameter)
pub fn apply_expand_filtering(
    url: &str,
    include_all_fields: Option<bool>,
    additional_expand: Option<Vec<String>>,
) -> (String, Option<String>) {
    // For v1 search API, we still use expand parameter
    let expand_params = if include_all_fields.unwrap_or(false) {
        vec!["body.storage", "version", "space", "history", "metadata"]
    } else {
        vec!["body.storage", "version"]
    };

    let mut expand = expand_params.iter().map(|s| s.to_string()).collect::<Vec<_>>();

    if let Some(additional) = additional_expand {
        for param in additional {
            if !expand.contains(&param) {
                expand.push(param);
            }
        }
    }

    (url.to_string(), Some(expand.join(",")))
}
