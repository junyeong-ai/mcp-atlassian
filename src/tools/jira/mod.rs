use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use crate::config::Config;
use crate::tools::ToolHandler;
use crate::utils::http_utils::{create_atlassian_client, create_auth_header};

pub mod field_filtering;
use field_filtering::apply_field_filtering;

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
        let issue_key = args["issue_key"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing issue_key"))?;

        let include_all_fields = args["include_all_fields"].as_bool();
        let additional_fields = args["additional_fields"].as_array()
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect());

        let client = create_atlassian_client(config);
        let base_url = format!(
            "{}/rest/api/3/issue/{}",
            config.get_atlassian_base_url(),
            issue_key
        );

        let url = apply_field_filtering(&base_url, include_all_fields, additional_fields);

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
        let jql = args["jql"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing jql"))?;
        let limit = args["limit"].as_u64().unwrap_or(10);

        // Apply project filter if configured and not already in JQL
        let final_jql = if !config.jira_projects_filter.is_empty() {
            let jql_lower = jql.to_lowercase();
            // Check if JQL already contains project condition
            if jql_lower.contains("project ") || jql_lower.contains("project=") || jql_lower.contains("project in") {
                // User explicitly specified project, use their JQL as-is
                jql.to_string()
            } else {
                // Add project filter
                let projects = config.jira_projects_filter
                    .iter()
                    .map(|p| format!("\"{}\"", p))
                    .collect::<Vec<_>>()
                    .join(",");
                format!("project IN ({}) AND ({})", projects, jql)
            }
        } else {
            jql.to_string()
        };

        let include_all_fields = args["include_all_fields"].as_bool();
        let additional_fields = args["additional_fields"].as_array()
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect());

        let client = create_atlassian_client(config);
        let base_url = config.get_atlassian_base_url();
        let url = format!(
            "{}/rest/api/3/search/jql",
            base_url
        );

        tracing::debug!("Jira search URL (before filtering): {}", url);

        // Build query with field filtering
        let mut query_params = vec![
            ("jql".to_string(), final_jql),
            ("maxResults".to_string(), limit.to_string()),
        ];

        // Add field filtering if not requesting all fields
        if !include_all_fields.unwrap_or(false) {
            let mut field_config = field_filtering::FieldConfiguration::from_env();
            if let Some(additional) = additional_fields {
                field_config = field_config.with_additional_fields(additional);
            }
            let selector = field_filtering::FieldSelector::from_config(&field_config);
            if let Some(fields) = selector.to_query_param() {
                query_params.push(("fields".to_string(), fields));
                // Exclude heavy data like renderedFields to minimize response size
                query_params.push(("expand".to_string(), "-renderedFields".to_string()));
            }
        }

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
        let include_all_fields = args["include_all_fields"].as_bool();
        let additional_fields = args["additional_fields"].as_array()
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect());

        let client = create_atlassian_client(config);
        let base_url = format!(
            "{}/rest/api/3/issue",
            config.get_atlassian_base_url()
        );

        let url = apply_field_filtering(&base_url, include_all_fields, additional_fields);

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
        let issue_key = args["issue_key"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing issue_key"))?;

        let include_all_fields = args["include_all_fields"].as_bool();
        let additional_fields = args["additional_fields"].as_array()
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect());

        let client = create_atlassian_client(config);
        let base_url = format!(
            "{}/rest/api/3/issue/{}",
            config.get_atlassian_base_url(),
            issue_key
        );

        let url = apply_field_filtering(&base_url, include_all_fields, additional_fields);

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
        let issue_key = args["issue_key"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing issue_key"))?;
        let comment = args["comment"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing comment"))?;

        let include_all_fields = args["include_all_fields"].as_bool();
        let additional_fields = args["additional_fields"].as_array()
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect());

        let client = create_atlassian_client(config);
        let base_url = format!(
            "{}/rest/api/3/issue/{}/comment",
            config.get_atlassian_base_url(),
            issue_key
        );

        let url = apply_field_filtering(&base_url, include_all_fields, additional_fields);

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
        let issue_key = args["issue_key"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing issue_key"))?;
        let transition_id = args["transition_id"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing transition_id"))?;

        let include_all_fields = args["include_all_fields"].as_bool();
        let additional_fields = args["additional_fields"].as_array()
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect());

        let client = create_atlassian_client(config);
        let base_url = format!(
            "{}/rest/api/3/issue/{}/transitions",
            config.get_atlassian_base_url(),
            issue_key
        );

        let url = apply_field_filtering(&base_url, include_all_fields, additional_fields);

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
        let issue_key = args["issue_key"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing issue_key"))?;

        let include_all_fields = args["include_all_fields"].as_bool();
        let additional_fields = args["additional_fields"].as_array()
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect());

        let client = create_atlassian_client(config);
        let base_url = format!(
            "{}/rest/api/3/issue/{}/transitions",
            config.get_atlassian_base_url(),
            issue_key
        );

        let url = apply_field_filtering(&base_url, include_all_fields, additional_fields);

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

