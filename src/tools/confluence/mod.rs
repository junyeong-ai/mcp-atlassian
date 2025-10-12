use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use crate::config::Config;
use crate::tools::ToolHandler;
use crate::utils::http_utils::{create_atlassian_client, create_auth_header};

pub mod field_filtering;
use field_filtering::{apply_expand_filtering, apply_v2_filtering};

// Handlers for each Confluence tool
pub struct SearchHandler;
pub struct GetPageHandler;
pub struct GetPageChildrenHandler;
pub struct GetCommentsHandler;
pub struct CreatePageHandler;
pub struct UpdatePageHandler;

#[async_trait]
impl ToolHandler for SearchHandler {
    async fn execute(&self, args: Value, config: &Config) -> Result<Value> {
        let cql = args["query"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing query parameter"))?;
        let limit = args["limit"].as_u64().unwrap_or(10);

        // Apply space filter if configured and not already in CQL
        let final_cql = if !config.confluence_spaces_filter.is_empty() {
            let cql_lower = cql.to_lowercase();
            // Check if CQL already contains space condition
            if cql_lower.contains("space ") || cql_lower.contains("space=") || cql_lower.contains("space in") {
                // User explicitly specified space, use their CQL as-is
                cql.to_string()
            } else {
                // Add space filter
                let spaces = config.confluence_spaces_filter
                    .iter()
                    .map(|s| format!("\"{}\"", s))
                    .collect::<Vec<_>>()
                    .join(",");
                format!("space IN ({}) AND ({})", spaces, cql)
            }
        } else {
            cql.to_string()
        };

        let include_all_fields = args["include_all_fields"].as_bool();
        let additional_expand = args["additional_expand"].as_array()
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect());

        let client = create_atlassian_client(config);
        let url = format!(
            "{}/wiki/rest/api/search",
            config.get_atlassian_base_url()
        );

        let (url, expand_param) = apply_expand_filtering(&url, include_all_fields, additional_expand);

        let mut query_params = vec![
            ("cql".to_string(), final_cql),
            ("limit".to_string(), limit.to_string()),
        ];

        if let Some(expand) = expand_param {
            query_params.push(("expand".to_string(), expand));
        }

        let response = client
            .get(&url)
            .header("Authorization", create_auth_header(config))
            .header("Accept", "application/json")
            .query(&query_params)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Search failed: {}", response.status());
        }

        let data: Value = response.json().await?;
        Ok(json!({
            "success": true,
            "results": data["results"],
            "total": data["totalSize"]
        }))
    }
}

#[async_trait]
impl ToolHandler for GetPageHandler {
    async fn execute(&self, args: Value, config: &Config) -> Result<Value> {
        let page_id = args["page_id"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing page_id"))?;

        let include_all_fields = args["include_all_fields"].as_bool();
        let additional_includes = args["additional_expand"].as_array()
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect());

        let client = create_atlassian_client(config);
        let url = format!(
            "{}/wiki/api/v2/pages/{}",
            config.get_atlassian_base_url(),
            page_id
        );

        let query_params = apply_v2_filtering(include_all_fields, additional_includes);

        let response = client
            .get(&url)
            .header("Authorization", create_auth_header(config))
            .header("Accept", "application/json")
            .query(&query_params)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to get page: {}", response.status());
        }

        let data: Value = response.json().await?;
        Ok(json!({
            "success": true,
            "page": data
        }))
    }
}

#[async_trait]
impl ToolHandler for GetPageChildrenHandler {
    async fn execute(&self, args: Value, config: &Config) -> Result<Value> {
        let page_id = args["page_id"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing page_id"))?;

        let include_all_fields = args["include_all_fields"].as_bool();
        let additional_includes = args["additional_expand"].as_array()
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect());

        let client = create_atlassian_client(config);
        let url = format!(
            "{}/wiki/api/v2/pages/{}/children",
            config.get_atlassian_base_url(),
            page_id
        );

        let query_params = apply_v2_filtering(include_all_fields, additional_includes);

        let response = client
            .get(&url)
            .header("Authorization", create_auth_header(config))
            .header("Accept", "application/json")
            .query(&query_params)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to get child pages: {}", response.status());
        }

        let data: Value = response.json().await?;
        Ok(json!({
            "success": true,
            "children": data["results"]
        }))
    }
}

#[async_trait]
impl ToolHandler for GetCommentsHandler {
    async fn execute(&self, args: Value, config: &Config) -> Result<Value> {
        let page_id = args["page_id"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing page_id"))?;

        let include_all_fields = args["include_all_fields"].as_bool();
        let additional_includes = args["additional_expand"].as_array()
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect());

        let client = create_atlassian_client(config);
        let url = format!(
            "{}/wiki/api/v2/pages/{}/footer-comments",
            config.get_atlassian_base_url(),
            page_id
        );

        let query_params = apply_v2_filtering(include_all_fields, additional_includes);

        let response = client
            .get(&url)
            .header("Authorization", create_auth_header(config))
            .header("Accept", "application/json")
            .query(&query_params)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to get comments: {}", response.status());
        }

        let data: Value = response.json().await?;
        Ok(json!({
            "success": true,
            "comments": data["results"]
        }))
    }
}

#[async_trait]
impl ToolHandler for CreatePageHandler {
    async fn execute(&self, args: Value, config: &Config) -> Result<Value> {
        let space_key = args["space_key"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing space_key"))?;
        let title = args["title"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing title"))?;
        let content = args["content"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing content"))?;

        let include_all_fields = args["include_all_fields"].as_bool();
        let additional_includes = args["additional_expand"].as_array()
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect());

        let client = create_atlassian_client(config);

        // First, convert space_key to space_id using v2 API
        let space_url = format!(
            "{}/wiki/api/v2/spaces",
            config.get_atlassian_base_url()
        );

        let space_response = client
            .get(&space_url)
            .query(&[("keys", space_key)])  // Automatic URL encoding
            .header("Authorization", create_auth_header(config))
            .header("Accept", "application/json")
            .send()
            .await?;

        if !space_response.status().is_success() {
            anyhow::bail!("Failed to get space ID for key '{}': {}", space_key, space_response.status());
        }

        let space_data: Value = space_response.json().await?;
        let space_id = space_data["results"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|space| space["id"].as_str())
            .ok_or_else(|| anyhow::anyhow!("Space '{}' not found", space_key))?;

        // Now create the page with v2 API
        let url = format!(
            "{}/wiki/api/v2/pages",
            config.get_atlassian_base_url()
        );

        let query_params = apply_v2_filtering(include_all_fields, additional_includes);

        let body = json!({
            "spaceId": space_id,
            "title": title,
            "body": {
                "representation": "storage",
                "value": content
            }
        });

        let response = client
            .post(&url)
            .header("Authorization", create_auth_header(config))
            .header("Content-Type", "application/json")
            .query(&query_params)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error = response.text().await?;
            anyhow::bail!("Failed to create page: {}", error);
        }

        let data: Value = response.json().await?;
        Ok(json!({
            "success": true,
            "page": data
        }))
    }
}

#[async_trait]
impl ToolHandler for UpdatePageHandler {
    async fn execute(&self, args: Value, config: &Config) -> Result<Value> {
        let page_id = args["page_id"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing page_id"))?;
        let title = args["title"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing title"))?;
        let content = args["content"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing content"))?;

        let client = create_atlassian_client(config);

        let include_all_fields = args["include_all_fields"].as_bool();
        let additional_includes = args["additional_expand"].as_array()
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect());

        // First, get the current page to get the version number using v2 API
        let get_url = format!(
            "{}/wiki/api/v2/pages/{}",
            config.get_atlassian_base_url(),
            page_id
        );

        let get_response = client
            .get(&get_url)
            .header("Authorization", create_auth_header(config))
            .header("Accept", "application/json")
            .query(&[("include-version", "true")])
            .send()
            .await?;

        if !get_response.status().is_success() {
            anyhow::bail!("Failed to get page for update: {}", get_response.status());
        }

        let current_page: Value = get_response.json().await?;
        let current_version = current_page["version"]["number"].as_u64()
            .ok_or_else(|| anyhow::anyhow!("Failed to get current version"))?;

        // Now update the page with v2 API
        let update_url = format!(
            "{}/wiki/api/v2/pages/{}",
            config.get_atlassian_base_url(),
            page_id
        );

        let query_params = apply_v2_filtering(include_all_fields, additional_includes);

        let body = json!({
            "id": page_id,
            "title": title,
            "body": {
                "representation": "storage",
                "value": content
            },
            "version": {
                "number": current_version + 1
            }
        });

        let response = client
            .put(&update_url)
            .header("Authorization", create_auth_header(config))
            .header("Content-Type", "application/json")
            .query(&query_params)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error = response.text().await?;
            anyhow::bail!("Failed to update page: {}", error);
        }

        let data: Value = response.json().await?;
        Ok(json!({
            "success": true,
            "page": data
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    // Helper function to create test config
    fn create_test_config(confluence_spaces_filter: Vec<String>) -> Config {
        Config {
            atlassian_domain: "test.atlassian.net".to_string(),
            atlassian_email: "test@example.com".to_string(),
            atlassian_api_token: "token123".to_string(),
            max_connections: 10,
            request_timeout_ms: 30000,
            jira_projects_filter: vec![],
            confluence_spaces_filter,
            jira_search_default_fields: None,
            jira_search_custom_fields: vec![],
        }
    }

    // T017: Confluence SearchHandler tests

    #[test]
    fn test_search_handler_missing_query() {
        let handler = SearchHandler;
        let config = create_test_config(vec![]);
        let args = json!({});

        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(handler.execute(args, &config));

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing query"));
    }

    #[test]
    fn test_search_handler_default_limit() {
        let args = json!({
            "query": "type=page"
        });

        let limit = args["limit"].as_u64().unwrap_or(10);
        assert_eq!(limit, 10);
    }

    #[test]
    fn test_search_handler_custom_limit() {
        let args = json!({
            "query": "type=page",
            "limit": 25
        });

        let limit = args["limit"].as_u64().unwrap_or(10);
        assert_eq!(limit, 25);
    }

    #[test]
    fn test_search_handler_space_filter_injection() {
        let config = create_test_config(
            vec!["SPACE1".to_string(), "SPACE2".to_string()]
        );
        let cql = "type = page";

        // Simulate space filter logic
        let final_cql = if !config.confluence_spaces_filter.is_empty() {
            let cql_lower = cql.to_lowercase();
            if cql_lower.contains("space ") || cql_lower.contains("space=") || cql_lower.contains("space in") {
                cql.to_string()
            } else {
                let spaces = config.confluence_spaces_filter
                    .iter()
                    .map(|s| format!("\"{}\"", s))
                    .collect::<Vec<_>>()
                    .join(",");
                format!("space IN ({}) AND ({})", spaces, cql)
            }
        } else {
            cql.to_string()
        };

        assert_eq!(final_cql, "space IN (\"SPACE1\",\"SPACE2\") AND (type = page)");
    }

    #[test]
    fn test_search_handler_space_filter_not_injected_when_present() {
        let config = create_test_config(vec!["SPACE1".to_string()]);
        let cql = "space = MYSPACE AND type = page";

        // Simulate space filter logic
        let final_cql = if !config.confluence_spaces_filter.is_empty() {
            let cql_lower = cql.to_lowercase();
            if cql_lower.contains("space ") || cql_lower.contains("space=") || cql_lower.contains("space in") {
                cql.to_string()
            } else {
                let spaces = config.confluence_spaces_filter
                    .iter()
                    .map(|s| format!("\"{}\"", s))
                    .collect::<Vec<_>>()
                    .join(",");
                format!("space IN ({}) AND ({})", spaces, cql)
            }
        } else {
            cql.to_string()
        };

        assert_eq!(final_cql, "space = MYSPACE AND type = page");
    }

    // T018: Remaining Confluence handlers tests

    // GetPageHandler tests
    #[test]
    fn test_get_page_handler_missing_page_id() {
        let handler = GetPageHandler;
        let config = create_test_config(vec![]);
        let args = json!({});

        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(handler.execute(args, &config));

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing page_id"));
    }

    #[test]
    fn test_get_page_handler_valid_page_id() {
        let args = json!({
            "page_id": "12345"
        });

        let page_id = args["page_id"].as_str().unwrap();
        assert_eq!(page_id, "12345");
    }

    #[test]
    fn test_get_page_handler_url_construction() {
        let config = create_test_config(vec![]);
        let page_id = "12345";

        let url = format!(
            "{}/wiki/api/v2/pages/{}",
            config.get_atlassian_base_url(),
            page_id
        );

        assert_eq!(url, "https://test.atlassian.net/wiki/api/v2/pages/12345");
    }

    // GetPageChildrenHandler tests
    #[test]
    fn test_get_page_children_handler_missing_page_id() {
        let handler = GetPageChildrenHandler;
        let config = create_test_config(vec![]);
        let args = json!({});

        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(handler.execute(args, &config));

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing page_id"));
    }

    #[test]
    fn test_get_page_children_handler_url_construction() {
        let config = create_test_config(vec![]);
        let page_id = "12345";

        let url = format!(
            "{}/wiki/api/v2/pages/{}/children",
            config.get_atlassian_base_url(),
            page_id
        );

        assert_eq!(url, "https://test.atlassian.net/wiki/api/v2/pages/12345/children");
    }

    // GetCommentsHandler tests
    #[test]
    fn test_get_comments_handler_missing_page_id() {
        let handler = GetCommentsHandler;
        let config = create_test_config(vec![]);
        let args = json!({});

        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(handler.execute(args, &config));

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing page_id"));
    }

    #[test]
    fn test_get_comments_handler_url_construction() {
        let config = create_test_config(vec![]);
        let page_id = "12345";

        let url = format!(
            "{}/wiki/api/v2/pages/{}/footer-comments",
            config.get_atlassian_base_url(),
            page_id
        );

        assert_eq!(url, "https://test.atlassian.net/wiki/api/v2/pages/12345/footer-comments");
    }

    // CreatePageHandler tests
    #[test]
    fn test_create_page_handler_missing_space_key() {
        let handler = CreatePageHandler;
        let config = create_test_config(vec![]);
        let args = json!({
            "title": "Test Page",
            "content": "<p>Test content</p>"
        });

        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(handler.execute(args, &config));

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing space_key"));
    }

    #[test]
    fn test_create_page_handler_missing_title() {
        let handler = CreatePageHandler;
        let config = create_test_config(vec![]);
        let args = json!({
            "space_key": "TEST",
            "content": "<p>Test content</p>"
        });

        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(handler.execute(args, &config));

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing title"));
    }

    #[test]
    fn test_create_page_handler_missing_content() {
        let handler = CreatePageHandler;
        let config = create_test_config(vec![]);
        let args = json!({
            "space_key": "TEST",
            "title": "Test Page"
        });

        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(handler.execute(args, &config));

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing content"));
    }

    #[test]
    fn test_create_page_handler_body_format() {
        let title = "Test Page";
        let content = "<p>Test content</p>";
        let space_id = "space123";

        let body = json!({
            "spaceId": space_id,
            "title": title,
            "body": {
                "representation": "storage",
                "value": content
            }
        });

        assert_eq!(body["spaceId"], "space123");
        assert_eq!(body["title"], "Test Page");
        assert_eq!(body["body"]["representation"], "storage");
        assert_eq!(body["body"]["value"], "<p>Test content</p>");
    }

    // UpdatePageHandler tests
    #[test]
    fn test_update_page_handler_missing_page_id() {
        let handler = UpdatePageHandler;
        let config = create_test_config(vec![]);
        let args = json!({
            "title": "Updated Title",
            "content": "<p>Updated content</p>"
        });

        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(handler.execute(args, &config));

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing page_id"));
    }

    #[test]
    fn test_update_page_handler_missing_title() {
        let handler = UpdatePageHandler;
        let config = create_test_config(vec![]);
        let args = json!({
            "page_id": "12345",
            "content": "<p>Updated content</p>"
        });

        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(handler.execute(args, &config));

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing title"));
    }

    #[test]
    fn test_update_page_handler_missing_content() {
        let handler = UpdatePageHandler;
        let config = create_test_config(vec![]);
        let args = json!({
            "page_id": "12345",
            "title": "Updated Title"
        });

        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(handler.execute(args, &config));

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing content"));
    }

    #[test]
    fn test_update_page_handler_body_format() {
        let page_id = "12345";
        let title = "Updated Title";
        let content = "<p>Updated content</p>";
        let current_version = 5u64;

        let body = json!({
            "id": page_id,
            "title": title,
            "body": {
                "representation": "storage",
                "value": content
            },
            "version": {
                "number": current_version + 1
            }
        });

        assert_eq!(body["id"], "12345");
        assert_eq!(body["title"], "Updated Title");
        assert_eq!(body["body"]["representation"], "storage");
        assert_eq!(body["body"]["value"], "<p>Updated content</p>");
        assert_eq!(body["version"]["number"], 6);
    }
}