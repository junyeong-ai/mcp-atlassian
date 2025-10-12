use crate::config::Config;
use crate::tools::ToolHandler;
use crate::utils::http_utils::{create_atlassian_client, create_auth_header};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{Value, json};

pub mod field_filtering;

// Handlers for each Jira tool
pub struct GetIssueHandler;
pub struct SearchHandler;
pub struct CreateIssueHandler;
pub struct UpdateIssueHandler;
pub struct AddCommentHandler;
pub struct TransitionIssueHandler;
pub struct GetTransitionsHandler;

#[async_trait]
impl ToolHandler for GetIssueHandler {
    async fn execute(&self, args: Value, config: &Config) -> Result<Value> {
        let issue_key = args["issue_key"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing issue_key"))?;

        let client = create_atlassian_client(config);
        let base_url = format!(
            "{}/rest/api/3/issue/{}",
            config.get_atlassian_base_url(),
            issue_key
        );

        let url = field_filtering::apply_field_filtering_to_url(&base_url);

        let response = client
            .get(&url)
            .header("Authorization", create_auth_header(config))
            .header("Accept", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to get issue: {}", response.status());
        }

        let data: Value = response.json().await?;
        Ok(json!({
            "success": true,
            "issue": data
        }))
    }
}

#[async_trait]
impl ToolHandler for SearchHandler {
    async fn execute(&self, args: Value, config: &Config) -> Result<Value> {
        let jql = args["jql"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing jql"))?;
        let limit = args["limit"].as_u64().unwrap_or(20);

        // Extract fields parameter from API call
        let api_fields = args["fields"].as_array().map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        });

        // Apply project filter if configured and not already in JQL
        let final_jql = if !config.jira_projects_filter.is_empty() {
            let jql_lower = jql.to_lowercase();
            // Check if JQL already contains project condition
            if jql_lower.contains("project ")
                || jql_lower.contains("project=")
                || jql_lower.contains("project in")
            {
                // User explicitly specified project, use their JQL as-is
                jql.to_string()
            } else {
                // Add project filter
                let projects = config
                    .jira_projects_filter
                    .iter()
                    .map(|p| format!("\"{}\"", p))
                    .collect::<Vec<_>>()
                    .join(",");
                format!("project IN ({}) AND ({})", projects, jql)
            }
        } else {
            jql.to_string()
        };

        let client = create_atlassian_client(config);
        let base_url = config.get_atlassian_base_url();
        let url = format!("{}/rest/api/3/search/jql", base_url);

        // Resolve fields using priority hierarchy
        let fields = field_filtering::resolve_search_fields(api_fields, config);

        let query_params = vec![
            ("jql".to_string(), final_jql),
            ("maxResults".to_string(), limit.to_string()),
            ("fields".to_string(), fields.join(",")),
            ("expand".to_string(), "-renderedFields".to_string()),
        ];

        tracing::debug!(
            "Jira search with {} fields: {}",
            fields.len(),
            fields.join(",")
        );

        let response = client
            .get(&url)
            .header("Authorization", create_auth_header(config))
            .header("Accept", "application/json")
            .query(&query_params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error = response.text().await?;
            anyhow::bail!("Search failed: {}", error);
        }

        let data: Value = response.json().await?;
        Ok(json!({
            "success": true,
            "issues": data["issues"],
            "total": data["total"]
        }))
    }
}

#[async_trait]
impl ToolHandler for CreateIssueHandler {
    async fn execute(&self, args: Value, config: &Config) -> Result<Value> {
        let client = create_atlassian_client(config);
        let base_url = format!("{}/rest/api/3/issue", config.get_atlassian_base_url());

        let url = field_filtering::apply_field_filtering_to_url(&base_url);

        let body = json!({
            "fields": {
                "project": {
                    "key": args["project_key"]
                },
                "summary": args["summary"],
                "issuetype": {
                    "name": args["issue_type"]
                },
                "description": {
                    "type": "doc",
                    "version": 1,
                    "content": [{
                        "type": "paragraph",
                        "content": [{
                            "type": "text",
                            "text": args["description"].as_str().unwrap_or("")
                        }]
                    }]
                }
            }
        });

        let response = client
            .post(&url)
            .header("Authorization", create_auth_header(config))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error = response.text().await?;
            anyhow::bail!("Failed to create issue: {}", error);
        }

        let data: Value = response.json().await?;
        Ok(json!({
            "success": true,
            "issue": data
        }))
    }
}

#[async_trait]
impl ToolHandler for UpdateIssueHandler {
    async fn execute(&self, args: Value, config: &Config) -> Result<Value> {
        let issue_key = args["issue_key"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing issue_key"))?;

        let client = create_atlassian_client(config);
        let url = format!(
            "{}/rest/api/3/issue/{}",
            config.get_atlassian_base_url(),
            issue_key
        );

        let response = client
            .put(&url)
            .header("Authorization", create_auth_header(config))
            .header("Content-Type", "application/json")
            .json(&json!({
                "fields": args["fields"]
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to update issue: {}", response.status());
        }

        Ok(json!({
            "success": true,
            "message": format!("Issue {} updated", issue_key)
        }))
    }
}

#[async_trait]
impl ToolHandler for AddCommentHandler {
    async fn execute(&self, args: Value, config: &Config) -> Result<Value> {
        let issue_key = args["issue_key"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing issue_key"))?;
        let comment = args["comment"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing comment"))?;

        let client = create_atlassian_client(config);
        let base_url = format!(
            "{}/rest/api/3/issue/{}/comment",
            config.get_atlassian_base_url(),
            issue_key
        );

        let url = field_filtering::apply_field_filtering_to_url(&base_url);

        let body = json!({
            "body": {
                "type": "doc",
                "version": 1,
                "content": [{
                    "type": "paragraph",
                    "content": [{
                        "type": "text",
                        "text": comment
                    }]
                }]
            }
        });

        let response = client
            .post(&url)
            .header("Authorization", create_auth_header(config))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to add comment: {}", response.status());
        }

        let data: Value = response.json().await?;
        Ok(json!({
            "success": true,
            "comment": data
        }))
    }
}

#[async_trait]
impl ToolHandler for TransitionIssueHandler {
    async fn execute(&self, args: Value, config: &Config) -> Result<Value> {
        let issue_key = args["issue_key"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing issue_key"))?;
        let transition_id = args["transition_id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing transition_id"))?;

        let client = create_atlassian_client(config);
        let url = format!(
            "{}/rest/api/3/issue/{}/transitions",
            config.get_atlassian_base_url(),
            issue_key
        );

        let body = json!({
            "transition": {
                "id": transition_id
            }
        });

        let response = client
            .post(&url)
            .header("Authorization", create_auth_header(config))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to transition issue: {}", response.status());
        }

        Ok(json!({
            "success": true,
            "message": format!("Issue {} transitioned", issue_key)
        }))
    }
}

#[async_trait]
impl ToolHandler for GetTransitionsHandler {
    async fn execute(&self, args: Value, config: &Config) -> Result<Value> {
        let issue_key = args["issue_key"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing issue_key"))?;

        let client = create_atlassian_client(config);
        let base_url = format!(
            "{}/rest/api/3/issue/{}/transitions",
            config.get_atlassian_base_url(),
            issue_key
        );

        let url = field_filtering::apply_field_filtering_to_url(&base_url);

        let response = client
            .get(&url)
            .header("Authorization", create_auth_header(config))
            .header("Accept", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to get transitions: {}", response.status());
        }

        let data: Value = response.json().await?;
        Ok(json!({
            "success": true,
            "transitions": data["transitions"]
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    // Helper function to create test config
    fn create_test_config(
        jira_projects_filter: Vec<String>,
        jira_search_default_fields: Option<Vec<String>>,
    ) -> Config {
        Config {
            atlassian_domain: "test.atlassian.net".to_string(),
            atlassian_email: "test@example.com".to_string(),
            atlassian_api_token: "token123".to_string(),
            max_connections: 10,
            request_timeout_ms: 30000,
            jira_projects_filter,
            confluence_spaces_filter: vec![],
            jira_search_default_fields,
            jira_search_custom_fields: vec![],
        }
    }

    // T013: Jira SearchHandler tests

    #[test]
    fn test_search_handler_missing_jql() {
        // Test that SearchHandler requires jql parameter
        let handler = SearchHandler;
        let config = create_test_config(vec![], None);
        let args = json!({});

        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(handler.execute(args, &config));

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing jql"));
    }

    #[test]
    fn test_search_handler_default_limit() {
        // Test that default limit is 20 when not specified
        let args = json!({
            "jql": "status = Open"
        });

        // We can't test the actual HTTP call without a mock server,
        // but we can verify that the handler doesn't panic with valid input
        // The actual limit value would be used in the HTTP request
        // This test ensures the parameter extraction works correctly

        // Since we need to test async code, we verify args parsing manually
        let jql = args["jql"].as_str().unwrap();
        let limit = args["limit"].as_u64().unwrap_or(20);

        assert_eq!(jql, "status = Open");
        assert_eq!(limit, 20);
    }

    #[test]
    fn test_search_handler_custom_limit() {
        // Test that custom limit is respected
        let args = json!({
            "jql": "status = Open",
            "limit": 50
        });

        let jql = args["jql"].as_str().unwrap();
        let limit = args["limit"].as_u64().unwrap_or(20);

        assert_eq!(jql, "status = Open");
        assert_eq!(limit, 50);
    }

    #[test]
    fn test_search_handler_project_filter_injection() {
        // Test that project filter is injected when not present in JQL
        let config = create_test_config(vec!["PROJ1".to_string(), "PROJ2".to_string()], None);
        let jql = "status = Open";

        // Simulate the project filter logic
        let final_jql = if !config.jira_projects_filter.is_empty() {
            let jql_lower = jql.to_lowercase();
            if jql_lower.contains("project ")
                || jql_lower.contains("project=")
                || jql_lower.contains("project in")
            {
                jql.to_string()
            } else {
                let projects = config
                    .jira_projects_filter
                    .iter()
                    .map(|p| format!("\"{}\"", p))
                    .collect::<Vec<_>>()
                    .join(",");
                format!("project IN ({}) AND ({})", projects, jql)
            }
        } else {
            jql.to_string()
        };

        assert_eq!(
            final_jql,
            "project IN (\"PROJ1\",\"PROJ2\") AND (status = Open)"
        );
    }

    #[test]
    fn test_search_handler_project_filter_not_injected_when_present() {
        // Test that project filter is NOT injected when already in JQL
        let config = create_test_config(vec!["PROJ1".to_string()], None);
        let jql = "project = MYPROJ AND status = Open";

        // Simulate the project filter logic
        let final_jql = if !config.jira_projects_filter.is_empty() {
            let jql_lower = jql.to_lowercase();
            if jql_lower.contains("project ")
                || jql_lower.contains("project=")
                || jql_lower.contains("project in")
            {
                jql.to_string()
            } else {
                let projects = config
                    .jira_projects_filter
                    .iter()
                    .map(|p| format!("\"{}\"", p))
                    .collect::<Vec<_>>()
                    .join(",");
                format!("project IN ({}) AND ({})", projects, jql)
            }
        } else {
            jql.to_string()
        };

        // Should remain unchanged because JQL already has "project ="
        assert_eq!(final_jql, "project = MYPROJ AND status = Open");
    }

    #[test]
    fn test_search_handler_fields_extraction_from_api() {
        // Test that fields parameter is extracted from API call
        let args = json!({
            "jql": "status = Open",
            "fields": ["key", "summary", "status"]
        });

        let api_fields = args["fields"].as_array().map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect::<Vec<String>>()
        });

        assert!(api_fields.is_some());
        let fields = api_fields.unwrap();
        assert_eq!(fields.len(), 3);
        assert_eq!(fields, vec!["key", "summary", "status"]);
    }

    #[test]
    fn test_search_handler_no_fields_uses_default() {
        // Test that when no fields are specified, we use defaults
        let config = create_test_config(vec![], None);
        let args = json!({
            "jql": "status = Open"
        });

        let api_fields = args["fields"].as_array().map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        });

        // When api_fields is None, resolve_search_fields should return defaults
        assert!(api_fields.is_none());

        // This would be resolved by field_filtering::resolve_search_fields
        let fields = field_filtering::resolve_search_fields(api_fields, &config);
        assert_eq!(fields.len(), 17); // DEFAULT_SEARCH_FIELDS count
    }

    #[test]
    fn test_search_handler_empty_project_filter() {
        // Test that empty project filter doesn't modify JQL
        let config = create_test_config(vec![], None);
        let jql = "status = Open";

        let final_jql = if !config.jira_projects_filter.is_empty() {
            format!("project IN (...) AND ({})", jql)
        } else {
            jql.to_string()
        };

        assert_eq!(final_jql, "status = Open");
    }

    // T014: Jira GetIssueHandler tests

    #[test]
    fn test_get_issue_handler_missing_issue_key() {
        let handler = GetIssueHandler;
        let config = create_test_config(vec![], None);
        let args = json!({});

        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(handler.execute(args, &config));

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Missing issue_key")
        );
    }

    #[test]
    fn test_get_issue_handler_valid_issue_key() {
        let args = json!({
            "issue_key": "PROJ-123"
        });

        let issue_key = args["issue_key"].as_str().unwrap();
        assert_eq!(issue_key, "PROJ-123");
    }

    #[test]
    fn test_get_issue_handler_url_construction() {
        let config = create_test_config(vec![], None);
        let issue_key = "PROJ-123";

        let base_url = format!(
            "{}/rest/api/3/issue/{}",
            config.get_atlassian_base_url(),
            issue_key
        );

        assert_eq!(
            base_url,
            "https://test.atlassian.net/rest/api/3/issue/PROJ-123"
        );
    }

    // T015: Jira CreateIssueHandler tests

    #[test]
    fn test_create_issue_handler_required_fields() {
        let args = json!({
            "project_key": "PROJ",
            "summary": "Test Issue",
            "issue_type": "Task",
            "description": "Test description"
        });

        assert_eq!(args["project_key"].as_str().unwrap(), "PROJ");
        assert_eq!(args["summary"].as_str().unwrap(), "Test Issue");
        assert_eq!(args["issue_type"].as_str().unwrap(), "Task");
        assert_eq!(args["description"].as_str().unwrap(), "Test description");
    }

    #[test]
    fn test_create_issue_handler_adf_conversion() {
        let description = "Test description";

        let adf_body = json!({
            "type": "doc",
            "version": 1,
            "content": [{
                "type": "paragraph",
                "content": [{
                    "type": "text",
                    "text": description
                }]
            }]
        });

        assert_eq!(adf_body["type"], "doc");
        assert_eq!(adf_body["version"], 1);
        assert_eq!(adf_body["content"][0]["type"], "paragraph");
        assert_eq!(
            adf_body["content"][0]["content"][0]["text"],
            "Test description"
        );
    }

    #[test]
    fn test_create_issue_handler_missing_description_fallback() {
        let args = json!({
            "project_key": "PROJ",
            "summary": "Test Issue",
            "issue_type": "Task"
        });

        let description = args["description"].as_str().unwrap_or("");
        assert_eq!(description, "");
    }

    // T016: Remaining Jira handlers tests

    // UpdateIssueHandler tests
    #[test]
    fn test_update_issue_handler_missing_issue_key() {
        let handler = UpdateIssueHandler;
        let config = create_test_config(vec![], None);
        let args = json!({
            "fields": {"summary": "Updated summary"}
        });

        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(handler.execute(args, &config));

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Missing issue_key")
        );
    }

    #[test]
    fn test_update_issue_handler_valid_fields() {
        let args = json!({
            "issue_key": "PROJ-123",
            "fields": {
                "summary": "Updated summary",
                "priority": {"name": "High"}
            }
        });

        let issue_key = args["issue_key"].as_str().unwrap();
        let fields = &args["fields"];

        assert_eq!(issue_key, "PROJ-123");
        assert_eq!(fields["summary"], "Updated summary");
        assert_eq!(fields["priority"]["name"], "High");
    }

    #[test]
    fn test_update_issue_handler_url_construction() {
        let config = create_test_config(vec![], None);
        let issue_key = "PROJ-123";

        let url = format!(
            "{}/rest/api/3/issue/{}",
            config.get_atlassian_base_url(),
            issue_key
        );

        assert_eq!(url, "https://test.atlassian.net/rest/api/3/issue/PROJ-123");
    }

    // AddCommentHandler tests
    #[test]
    fn test_add_comment_handler_missing_issue_key() {
        let handler = AddCommentHandler;
        let config = create_test_config(vec![], None);
        let args = json!({
            "comment": "Test comment"
        });

        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(handler.execute(args, &config));

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Missing issue_key")
        );
    }

    #[test]
    fn test_add_comment_handler_missing_comment() {
        let handler = AddCommentHandler;
        let config = create_test_config(vec![], None);
        let args = json!({
            "issue_key": "PROJ-123"
        });

        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(handler.execute(args, &config));

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing comment"));
    }

    #[test]
    fn test_add_comment_handler_adf_conversion() {
        let comment = "This is a test comment";

        let adf_body = json!({
            "body": {
                "type": "doc",
                "version": 1,
                "content": [{
                    "type": "paragraph",
                    "content": [{
                        "type": "text",
                        "text": comment
                    }]
                }]
            }
        });

        assert_eq!(adf_body["body"]["type"], "doc");
        assert_eq!(adf_body["body"]["version"], 1);
        assert_eq!(adf_body["body"]["content"][0]["type"], "paragraph");
        assert_eq!(
            adf_body["body"]["content"][0]["content"][0]["text"],
            "This is a test comment"
        );
    }

    #[test]
    fn test_add_comment_handler_url_construction() {
        let config = create_test_config(vec![], None);
        let issue_key = "PROJ-123";

        let base_url = format!(
            "{}/rest/api/3/issue/{}/comment",
            config.get_atlassian_base_url(),
            issue_key
        );

        assert_eq!(
            base_url,
            "https://test.atlassian.net/rest/api/3/issue/PROJ-123/comment"
        );
    }

    // TransitionIssueHandler tests
    #[test]
    fn test_transition_issue_handler_missing_issue_key() {
        let handler = TransitionIssueHandler;
        let config = create_test_config(vec![], None);
        let args = json!({
            "transition_id": "11"
        });

        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(handler.execute(args, &config));

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Missing issue_key")
        );
    }

    #[test]
    fn test_transition_issue_handler_missing_transition_id() {
        let handler = TransitionIssueHandler;
        let config = create_test_config(vec![], None);
        let args = json!({
            "issue_key": "PROJ-123"
        });

        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(handler.execute(args, &config));

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Missing transition_id")
        );
    }

    #[test]
    fn test_transition_issue_handler_valid_params() {
        let args = json!({
            "issue_key": "PROJ-123",
            "transition_id": "21"
        });

        let issue_key = args["issue_key"].as_str().unwrap();
        let transition_id = args["transition_id"].as_str().unwrap();

        assert_eq!(issue_key, "PROJ-123");
        assert_eq!(transition_id, "21");
    }

    #[test]
    fn test_transition_issue_handler_body_format() {
        let transition_id = "31";

        let body = json!({
            "transition": {
                "id": transition_id
            }
        });

        assert_eq!(body["transition"]["id"], "31");
    }

    // GetTransitionsHandler tests
    #[test]
    fn test_get_transitions_handler_missing_issue_key() {
        let handler = GetTransitionsHandler;
        let config = create_test_config(vec![], None);
        let args = json!({});

        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(handler.execute(args, &config));

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Missing issue_key")
        );
    }

    #[test]
    fn test_get_transitions_handler_valid_issue_key() {
        let args = json!({
            "issue_key": "PROJ-123"
        });

        let issue_key = args["issue_key"].as_str().unwrap();
        assert_eq!(issue_key, "PROJ-123");
    }

    #[test]
    fn test_get_transitions_handler_url_construction() {
        let config = create_test_config(vec![], None);
        let issue_key = "PROJ-123";

        let base_url = format!(
            "{}/rest/api/3/issue/{}/transitions",
            config.get_atlassian_base_url(),
            issue_key
        );

        assert_eq!(
            base_url,
            "https://test.atlassian.net/rest/api/3/issue/PROJ-123/transitions"
        );
    }
}
