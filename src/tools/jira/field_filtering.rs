/// Enhanced default fields for Jira search (optimized for token efficiency)
/// 17 fields providing comprehensive issue information without heavy content
pub const DEFAULT_SEARCH_FIELDS: &[&str] = &[
    // Identification
    "key",
    // Core Metadata
    "summary",
    "status",
    "priority",
    "issuetype",
    // People
    "assignee",
    "reporter",
    "creator",
    // Dates
    "created",
    "updated",
    "duedate",
    "resolutiondate",
    // Classification
    "project",
    "labels",
    "components",
    // Hierarchy
    "parent",
    "subtasks",
];

/// Resolves which fields to request for Jira search based on priority hierarchy:
/// 1. API-provided fields (highest priority - explicit user request)
/// 2. JIRA_SEARCH_DEFAULT_FIELDS env var (override built-in defaults completely)
/// 3. DEFAULT_SEARCH_FIELDS + JIRA_SEARCH_CUSTOM_FIELDS (built-in defaults + additions)
/// 4. DEFAULT_SEARCH_FIELDS only (fallback)
pub fn resolve_search_fields(
    api_fields: Option<Vec<String>>,
    config: &crate::config::Config,
) -> Vec<String> {
    // Priority 1: API-provided fields override everything
    if let Some(fields) = api_fields
        && !fields.is_empty()
    {
        tracing::debug!("Using {} fields from API parameters", fields.len());
        return fields;
    }

    // Priority 2: Environment variable completely replaces defaults
    if let Some(ref env_defaults) = config.jira_search_default_fields {
        tracing::debug!(
            "Using {} fields from JIRA_SEARCH_DEFAULT_FIELDS env var",
            env_defaults.len()
        );
        return env_defaults.clone();
    }

    // Priority 3 & 4: Built-in defaults, optionally extended with custom fields
    let mut fields: Vec<String> = DEFAULT_SEARCH_FIELDS
        .iter()
        .map(|s| s.to_string())
        .collect();

    if !config.jira_search_custom_fields.is_empty() {
        tracing::debug!(
            "Adding {} custom fields to {} default fields",
            config.jira_search_custom_fields.len(),
            fields.len()
        );
        fields.extend(config.jira_search_custom_fields.clone());
    }

    tracing::debug!("Resolved {} total fields for Jira search", fields.len());
    fields
}

/// Simple field filtering for non-search endpoints (GetIssue, CreateIssue, etc.)
/// These endpoints keep the old ESSENTIAL_FIELDS for backward compatibility
pub const ESSENTIAL_FIELDS: &[&str] = &[
    "key",
    "summary",
    "description",
    "issuetype",
    "status",
    "priority",
    "assignee",
    "reporter",
    "created",
    "updated",
    "project",
];

/// Helper function to apply field filtering to URLs for non-search endpoints
pub fn apply_field_filtering_to_url(base_url: &str) -> String {
    let fields = ESSENTIAL_FIELDS.join(",");

    let url_with_fields = if base_url.contains('?') {
        format!("{}&fields={}", base_url, fields)
    } else {
        format!("{}?fields={}", base_url, fields)
    };

    // Exclude heavy rendered fields
    format!("{}&expand=-renderedFields", url_with_fields)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    fn create_test_config(
        default_fields: Option<Vec<String>>,
        custom_fields: Vec<String>,
    ) -> Config {
        Config {
            atlassian_domain: "test.atlassian.net".to_string(),
            atlassian_email: "test@example.com".to_string(),
            atlassian_api_token: "token123".to_string(),
            request_timeout_ms: 30000,
            jira_projects_filter: vec![],
            confluence_spaces_filter: vec![],
            jira_search_default_fields: default_fields,
            jira_search_custom_fields: custom_fields,
            response_exclude_fields: None,
            base_url: "https://test.atlassian.net".to_string(),
        }
    }

    #[test]
    fn test_default_search_fields_count() {
        assert_eq!(DEFAULT_SEARCH_FIELDS.len(), 17);
    }

    #[test]
    fn test_default_fields_no_description() {
        assert!(!DEFAULT_SEARCH_FIELDS.contains(&"description"));
    }

    #[test]
    fn test_default_fields_no_id() {
        assert!(!DEFAULT_SEARCH_FIELDS.contains(&"id"));
    }

    #[test]
    fn test_resolve_priority_1_api_fields() {
        let config = create_test_config(
            Some(vec!["key".to_string()]),
            vec!["customfield_10015".to_string()],
        );

        let api_fields = Some(vec!["key".to_string(), "summary".to_string()]);
        let result = resolve_search_fields(api_fields, &config);

        // API fields override everything
        assert_eq!(result, vec!["key", "summary"]);
    }

    #[test]
    fn test_resolve_priority_2_env_override() {
        let config = create_test_config(
            Some(vec![
                "key".to_string(),
                "summary".to_string(),
                "status".to_string(),
            ]),
            vec!["customfield_10015".to_string()],
        );

        let result = resolve_search_fields(None, &config);

        // ENV override takes precedence, custom fields ignored
        assert_eq!(result, vec!["key", "summary", "status"]);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_resolve_priority_3_defaults_with_custom() {
        let config = create_test_config(
            None,
            vec![
                "customfield_10015".to_string(),
                "customfield_10016".to_string(),
            ],
        );

        let result = resolve_search_fields(None, &config);

        // 17 defaults + 2 custom = 19
        assert_eq!(result.len(), 19);
        assert!(result.contains(&"key".to_string()));
        assert!(result.contains(&"summary".to_string()));
        assert!(result.contains(&"customfield_10015".to_string()));
        assert!(result.contains(&"customfield_10016".to_string()));
    }

    #[test]
    fn test_resolve_priority_4_defaults_only() {
        let config = create_test_config(None, vec![]);

        let result = resolve_search_fields(None, &config);

        // Just the 17 default fields
        assert_eq!(result.len(), 17);
        assert!(result.contains(&"key".to_string()));
        assert!(result.contains(&"duedate".to_string()));
        assert!(result.contains(&"labels".to_string()));
        assert!(result.contains(&"parent".to_string()));
    }

    #[test]
    fn test_resolve_empty_api_fields_fallback() {
        let config = create_test_config(Some(vec!["key".to_string()]), vec![]);

        // Empty vec should be treated as "not provided"
        let result = resolve_search_fields(Some(vec![]), &config);

        // Falls back to env override
        assert_eq!(result, vec!["key"]);
    }

    #[test]
    fn test_new_fields_included() {
        let config = create_test_config(None, vec![]);
        let result = resolve_search_fields(None, &config);

        // Check new fields are included
        assert!(result.contains(&"duedate".to_string()));
        assert!(result.contains(&"resolutiondate".to_string()));
        assert!(result.contains(&"labels".to_string()));
        assert!(result.contains(&"components".to_string()));
        assert!(result.contains(&"parent".to_string()));
        assert!(result.contains(&"subtasks".to_string()));
    }

    // T025: Additional field filtering tests
    #[test]
    fn test_essential_fields_count() {
        assert_eq!(ESSENTIAL_FIELDS.len(), 11);
    }

    #[test]
    fn test_essential_fields_contains_description() {
        assert!(ESSENTIAL_FIELDS.contains(&"description"));
    }

    #[test]
    fn test_essential_fields_has_key_fields() {
        assert!(ESSENTIAL_FIELDS.contains(&"key"));
        assert!(ESSENTIAL_FIELDS.contains(&"summary"));
        assert!(ESSENTIAL_FIELDS.contains(&"status"));
        assert!(ESSENTIAL_FIELDS.contains(&"priority"));
    }

    #[test]
    fn test_apply_field_filtering_to_url_without_query_params() {
        let base_url = "https://test.atlassian.net/rest/api/3/issue/TEST-123";
        let result = apply_field_filtering_to_url(base_url);

        // Should add ? before fields
        assert!(result.contains("?fields="));
        // Should include expand parameter
        assert!(result.contains("&expand=-renderedFields"));
        // Should contain essential fields
        assert!(result.contains("key,summary,description"));
    }

    #[test]
    fn test_apply_field_filtering_to_url_with_query_params() {
        let base_url = "https://test.atlassian.net/rest/api/3/issue/TEST-123?foo=bar";
        let result = apply_field_filtering_to_url(base_url);

        // Should add & before fields (not ?)
        assert!(result.contains("&fields="));
        assert!(!result.contains("?fields="));
        // Should still have expand parameter
        assert!(result.contains("&expand=-renderedFields"));
        // Original param should be preserved
        assert!(result.contains("foo=bar"));
    }

    #[test]
    fn test_apply_field_filtering_url_structure() {
        let base_url = "https://test.atlassian.net/rest/api/3/issue/TEST-123";
        let result = apply_field_filtering_to_url(base_url);

        // Verify URL structure
        assert!(result.starts_with(base_url));
        assert_eq!(result.matches('?').count(), 1); // Exactly one ?
        assert_eq!(result.matches("&expand=-renderedFields").count(), 1); // Expand at end
    }

    #[test]
    fn test_default_search_fields_includes_all_categories() {
        let config = create_test_config(None, vec![]);
        let result = resolve_search_fields(None, &config);

        // Identification
        assert!(result.contains(&"key".to_string()));

        // Core Metadata
        assert!(result.contains(&"summary".to_string()));
        assert!(result.contains(&"status".to_string()));
        assert!(result.contains(&"priority".to_string()));
        assert!(result.contains(&"issuetype".to_string()));

        // People
        assert!(result.contains(&"assignee".to_string()));
        assert!(result.contains(&"reporter".to_string()));
        assert!(result.contains(&"creator".to_string()));

        // Dates
        assert!(result.contains(&"created".to_string()));
        assert!(result.contains(&"updated".to_string()));

        // Classification
        assert!(result.contains(&"project".to_string()));

        // Hierarchy
        assert!(result.contains(&"parent".to_string()));
        assert!(result.contains(&"subtasks".to_string()));
    }
}
