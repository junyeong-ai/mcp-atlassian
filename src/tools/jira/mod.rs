use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use crate::config::Config;
use crate::tools::ToolHandler;
use crate::utils::http_utils::{create_atlassian_client, create_auth_header};

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
        let issue_key = args["issue_key"].as_str()
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
        let jql = args["jql"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing jql"))?;
        let limit = args["limit"].as_u64().unwrap_or(20);

        // Extract fields parameter from API call
        let api_fields = args["fields"].as_array()
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect());

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

        tracing::debug!("Jira search with {} fields: {}", fields.len(), fields.join(","));

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
        let base_url = format!(
            "{}/rest/api/3/issue",
            config.get_atlassian_base_url()
        );

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
        let issue_key = args["issue_key"].as_str()
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
        let issue_key = args["issue_key"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing issue_key"))?;
        let comment = args["comment"].as_str()
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
        let issue_key = args["issue_key"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing issue_key"))?;
        let transition_id = args["transition_id"].as_str()
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
        let issue_key = args["issue_key"].as_str()
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

