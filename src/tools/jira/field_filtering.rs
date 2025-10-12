
/// Essential fields that are always included in Jira API responses
/// Note: For user fields (assignee, reporter, creator), we only request
/// essential properties to minimize data transfer
pub const ESSENTIAL_FIELDS: &[&str] = &[
    "id",
    "key",
    "summary",
    "description",
    "issuetype",
    "status",
    "priority",
    "assignee",
    "reporter",
    "creator",
    "created",
    "updated",
    "project",
];

/// Configuration for Jira field filtering
#[derive(Debug, Clone)]
pub struct FieldConfiguration {
    pub essential_fields: Vec<String>,
    pub custom_fields: Vec<String>,
    pub include_all: bool,
}

impl FieldConfiguration {
    /// Create a new field configuration from environment variables
    pub fn from_env() -> Self {
        let custom_fields_env = std::env::var("JIRA_CUSTOM_FIELDS").unwrap_or_default();
        let custom_fields: Vec<String> = if custom_fields_env.is_empty() {
            vec![]
        } else {
            custom_fields_env
                .split(',')
                .filter(|s| !s.is_empty())
                .map(|s| s.trim().to_string())
                .filter(|s| {
                    let valid = s.starts_with("customfield_");
                    if !valid && !s.is_empty() {
                        tracing::warn!("Invalid custom field name ignored: {}", s);
                    }
                    valid
                })
                .collect()
        };

        if !custom_fields.is_empty() {
            tracing::info!("Loaded {} custom fields: {:?}", custom_fields.len(), custom_fields);
        }

        Self {
            essential_fields: ESSENTIAL_FIELDS.iter().map(|s| s.to_string()).collect(),
            custom_fields,
            include_all: false,
        }
    }


    /// Get all fields to request
    pub fn get_fields(&self) -> Vec<String> {
        if self.include_all {
            return vec![];
        }

        let mut fields = self.essential_fields.clone();
        fields.extend(self.custom_fields.clone());
        fields
    }

    /// Override to include additional fields for a specific request
    pub fn with_additional_fields(&self, additional: Vec<String>) -> Self {
        let mut config = self.clone();
        for field in additional {
            if !config.essential_fields.contains(&field) && !config.custom_fields.contains(&field) {
                config.custom_fields.push(field);
            }
        }
        config
    }
}

/// Builds field query parameters for Jira API requests
#[derive(Debug, Clone)]
pub struct FieldSelector {
    fields: Vec<String>,
}

impl FieldSelector {
    /// Create a new field selector from configuration
    pub fn from_config(config: &FieldConfiguration) -> Self {
        Self {
            fields: config.get_fields(),
        }
    }


    /// Generate the query parameter for the fields
    pub fn to_query_param(&self) -> Option<String> {
        if self.fields.is_empty() {
            None
        } else {
            Some(self.fields.join(","))
        }
    }

}

/// Helper function to apply field filtering to a URL
pub fn apply_field_filtering(
    url: &str,
    include_all_fields: Option<bool>,
    additional_fields: Option<Vec<String>>,
) -> String {
    if include_all_fields.unwrap_or(false) {
        tracing::debug!("Field filtering disabled: include_all_fields=true");
        return url.to_string();
    }

    let mut config = FieldConfiguration::from_env();
    tracing::debug!("Loaded {} custom fields from environment", config.custom_fields.len());

    if let Some(additional) = additional_fields {
        tracing::debug!("Adding {} additional fields", additional.len());
        config = config.with_additional_fields(additional);
    }

    let selector = FieldSelector::from_config(&config);

    if let Some(fields) = selector.to_query_param() {
        let field_count = fields.split(',').count();
        tracing::debug!("Applying field filter with {} fields: {}", field_count, fields);

        // Build URL with field filtering
        let mut final_url = if url.contains('?') {
            format!("{}&fields={}", url, fields)
        } else {
            format!("{}?fields={}", url, fields)
        };

        // Add expand parameter to exclude renderedFields and other heavy data
        // This reduces the response size significantly
        final_url.push_str("&expand=-renderedFields");

        tracing::debug!("Final URL with optimizations: {}", final_url);
        final_url
    } else {
        tracing::debug!("No field filtering applied");
        url.to_string()
    }
}
