use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;

use crate::config::Config;
use crate::tools::ToolHandler;
use crate::tools::response_optimizer::ResponseOptimizer;
use crate::tools::{confluence, jira};

use super::types::{CallToolResult, Property, Tool as McpTool, ToolContent, ToolInputSchema};

pub struct RequestHandler {
    tools: HashMap<String, Arc<dyn ToolHandler>>,
    config: Arc<Config>,
    optimizer: Arc<ResponseOptimizer>,
}

impl RequestHandler {
    pub async fn new(config: Arc<Config>) -> Result<Self> {
        let mut tools: HashMap<String, Arc<dyn ToolHandler>> = HashMap::new();

        // Register Jira tools
        tools.insert(
            "jira_get_issue".to_string(),
            Arc::new(jira::GetIssueHandler),
        );
        tools.insert("jira_search".to_string(), Arc::new(jira::SearchHandler));
        tools.insert(
            "jira_create_issue".to_string(),
            Arc::new(jira::CreateIssueHandler),
        );
        tools.insert(
            "jira_update_issue".to_string(),
            Arc::new(jira::UpdateIssueHandler),
        );
        tools.insert(
            "jira_add_comment".to_string(),
            Arc::new(jira::AddCommentHandler),
        );
        tools.insert(
            "jira_update_comment".to_string(),
            Arc::new(jira::UpdateCommentHandler),
        );
        tools.insert(
            "jira_transition_issue".to_string(),
            Arc::new(jira::TransitionIssueHandler),
        );
        tools.insert(
            "jira_get_transitions".to_string(),
            Arc::new(jira::GetTransitionsHandler),
        );

        // Register Confluence tools
        tools.insert(
            "confluence_search".to_string(),
            Arc::new(confluence::SearchHandler),
        );
        tools.insert(
            "confluence_get_page".to_string(),
            Arc::new(confluence::GetPageHandler),
        );
        tools.insert(
            "confluence_get_page_children".to_string(),
            Arc::new(confluence::GetPageChildrenHandler),
        );
        tools.insert(
            "confluence_get_comments".to_string(),
            Arc::new(confluence::GetCommentsHandler),
        );
        tools.insert(
            "confluence_create_page".to_string(),
            Arc::new(confluence::CreatePageHandler),
        );
        tools.insert(
            "confluence_update_page".to_string(),
            Arc::new(confluence::UpdatePageHandler),
        );

        // Create response optimizer for field removal
        let optimizer = Arc::new(ResponseOptimizer::from_config(&config));

        Ok(Self {
            tools,
            config,
            optimizer,
        })
    }

    pub async fn list_tools(&self) -> Vec<McpTool> {
        let mut tool_list = Vec::new();

        for name in self.tools.keys() {
            tool_list.push(self.tool_to_mcp_tool(name, &self.config));
        }

        tool_list
    }

    pub async fn call_tool(
        &self,
        name: &str,
        arguments: Value,
        config: &Config,
    ) -> Result<CallToolResult> {
        let tool = self
            .tools
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", name))?;

        let mut result = tool.execute(arguments, config).await?;

        // Apply response optimization for GET operations only
        // CREATE/UPDATE operations already return minimal responses (Phase 3)
        let is_get_operation = matches!(
            name,
            "jira_get_issue"
                | "jira_search"
                | "jira_get_transitions"
                | "confluence_search"
                | "confluence_get_page"
                | "confluence_get_page_children"
                | "confluence_get_comments"
        );

        if is_get_operation {
            match self.optimizer.optimize(&mut result) {
                Ok(()) => {
                    tracing::debug!(tool = name, "Response optimization applied successfully");
                }
                Err(e) => {
                    tracing::warn!(
                        tool = name,
                        error = %e,
                        "Response optimization failed, returning unoptimized response"
                    );
                }
            }
        }

        // Convert result to tool content
        let content = if let Some(text) = result.as_str() {
            vec![ToolContent::Text {
                text: text.to_string(),
            }]
        } else {
            vec![ToolContent::Text {
                text: serde_json::to_string_pretty(&result)?,
            }]
        };

        Ok(CallToolResult { content })
    }

    fn create_string_prop(description: &str, _required: bool) -> Property {
        Property {
            property_type: json!("string"),
            description: Some(description.to_string()),
            default: None,
            enum_values: None,
        }
    }

    fn create_number_prop(description: &str, default: i32) -> Property {
        Property {
            property_type: json!("number"),
            description: Some(description.to_string()),
            default: Some(Value::Number(default.into())),
            enum_values: None,
        }
    }

    fn create_union_prop(description: &str, types: Vec<&str>) -> Property {
        Property {
            property_type: json!(types),
            description: Some(description.to_string()),
            default: None,
            enum_values: None,
        }
    }

    fn tool_to_mcp_tool(&self, name: &str, config: &Config) -> McpTool {
        let (description, properties, required) = match name {
            // Jira tools
            "jira_get_issue" => {
                let mut props = HashMap::new();
                props.insert(
                    "issue_key".to_string(),
                    Self::create_string_prop(
                        "Issue key (e.g., 'PROJECT-123'). Case-sensitive.",
                        true,
                    ),
                );
                (
                    "Get Jira issue by key",
                    props,
                    vec!["issue_key".to_string()],
                )
            }
            "jira_search" => {
                // Resolve the actual fields that will be used
                let resolved_fields = jira::field_filtering::resolve_search_fields(None, config);
                let fields_count = resolved_fields.len();
                let fields_list = resolved_fields.join(", ");

                let mut props = HashMap::new();
                props.insert("jql".to_string(), Self::create_string_prop("JQL query. Must include search condition before ORDER BY (e.g., 'project = KEY ORDER BY created DESC'). ORDER BY only works with orderable fields (dates, versions).", true));
                props.insert(
                    "limit".to_string(),
                    Self::create_number_prop("Maximum results (default: 20)", 20),
                );
                props.insert("fields".to_string(), Property {
                    property_type: json!("array"),
                    description: Some(format!(
                        "Optional: Array of field names to return. If not specified, returns {} default fields: {}\n\n\
                        To minimize tokens, specify only the fields you need (e.g., [\"key\",\"summary\",\"status\",\"assignee\"]).",
                        fields_count, fields_list
                    )),
                    default: None,
                    enum_values: None,
                });
                (
                    "Search Jira issues using JQL",
                    props,
                    vec!["jql".to_string()],
                )
            }
            "jira_create_issue" => {
                let mut props = HashMap::new();
                props.insert(
                    "project_key".to_string(),
                    Self::create_string_prop("Project key", true),
                );
                props.insert(
                    "summary".to_string(),
                    Self::create_string_prop("Issue summary", true),
                );
                props.insert(
                    "issue_type".to_string(),
                    Self::create_string_prop(
                        "Issue type name (e.g., 'Task', 'Bug', 'Story').",
                        true,
                    ),
                );
                props.insert(
                    "description".to_string(),
                    Self::create_union_prop(
                        "Issue description - accepts plain text (string, auto-converted to ADF) or ADF object",
                        vec!["string", "object"],
                    ),
                );
                (
                    "Create Jira issue",
                    props,
                    vec![
                        "project_key".to_string(),
                        "summary".to_string(),
                        "issue_type".to_string(),
                    ],
                )
            }
            "jira_update_issue" => {
                let mut props = HashMap::new();
                props.insert(
                    "issue_key".to_string(),
                    Self::create_string_prop("Issue key", true),
                );
                props.insert("fields".to_string(), Property {
                    property_type: json!("object"),
                    description: Some("Fields to update as JSON object (e.g., {\"summary\": \"New title\"}). Custom fields use 'customfield_*' format. The 'description' field accepts plain text (auto-converted to ADF) or ADF object.".to_string()),
                    default: None,
                    enum_values: None,
                });
                (
                    "Update Jira issue",
                    props,
                    vec!["issue_key".to_string(), "fields".to_string()],
                )
            }
            "jira_add_comment" => {
                let mut props = HashMap::new();
                props.insert(
                    "issue_key".to_string(),
                    Self::create_string_prop("Issue key", true),
                );
                props.insert(
                    "comment".to_string(),
                    Self::create_union_prop(
                        "Comment text - accepts plain text (string, auto-converted to ADF) or ADF object",
                        vec!["string", "object"],
                    ),
                );
                (
                    "Add comment to Jira issue",
                    props,
                    vec!["issue_key".to_string(), "comment".to_string()],
                )
            }
            "jira_update_comment" => {
                let mut props = HashMap::new();
                props.insert(
                    "issue_key".to_string(),
                    Self::create_string_prop("Issue key (e.g., 'PROJ-123')", true),
                );
                props.insert(
                    "comment_id".to_string(),
                    Self::create_string_prop(
                        "Comment ID to update (obtained from comment object's 'id' field)",
                        true,
                    ),
                );
                props.insert(
                    "body".to_string(),
                    Self::create_union_prop(
                        "Comment body - accepts plain text (string, auto-converted to ADF) or ADF object",
                        vec!["string", "object"],
                    ),
                );
                (
                    "Update an existing comment on a Jira issue with rich text formatting (ADF)",
                    props,
                    vec![
                        "issue_key".to_string(),
                        "comment_id".to_string(),
                        "body".to_string(),
                    ],
                )
            }
            "jira_transition_issue" => {
                let mut props = HashMap::new();
                props.insert(
                    "issue_key".to_string(),
                    Self::create_string_prop("Issue key", true),
                );
                props.insert("transition_id".to_string(), Self::create_string_prop("Transition ID. Get available transition IDs using jira_get_transitions for the issue's current status.", true));
                (
                    "Transition Jira issue status",
                    props,
                    vec!["issue_key".to_string(), "transition_id".to_string()],
                )
            }
            "jira_get_transitions" => {
                let mut props = HashMap::new();
                props.insert(
                    "issue_key".to_string(),
                    Self::create_string_prop("Issue key", true),
                );
                (
                    "Get Jira issue transitions",
                    props,
                    vec!["issue_key".to_string()],
                )
            }
            // Confluence tools
            "confluence_search" => {
                let mut props = HashMap::new();
                props.insert("query".to_string(), Self::create_string_prop("CQL query. Format: field operator value (e.g., 'type=page AND space=\"SPACE\"'). Use text ~ \"keyword\" for text search.", true));
                props.insert(
                    "limit".to_string(),
                    Self::create_number_prop("Max results", 10),
                );
                (
                    "Search Confluence using CQL",
                    props,
                    vec!["query".to_string()],
                )
            }
            "confluence_get_page" => {
                let mut props = HashMap::new();
                props.insert(
                    "page_id".to_string(),
                    Self::create_string_prop("Page ID", true),
                );
                (
                    "Get Confluence page by ID",
                    props,
                    vec!["page_id".to_string()],
                )
            }
            "confluence_get_page_children" => {
                let mut props = HashMap::new();
                props.insert(
                    "page_id".to_string(),
                    Self::create_string_prop("Page ID", true),
                );
                ("Get page child pages", props, vec!["page_id".to_string()])
            }
            "confluence_get_comments" => {
                let mut props = HashMap::new();
                props.insert(
                    "page_id".to_string(),
                    Self::create_string_prop("Page ID", true),
                );
                ("Get page comments", props, vec!["page_id".to_string()])
            }
            "confluence_create_page" => {
                let mut props = HashMap::new();
                props.insert(
                    "space_key".to_string(),
                    Self::create_string_prop("Space key", true),
                );
                props.insert(
                    "title".to_string(),
                    Self::create_string_prop("Page title", true),
                );
                props.insert(
                    "content".to_string(),
                    Self::create_string_prop("Page content in HTML storage format.", true),
                );
                props.insert(
                    "parent_id".to_string(),
                    Self::create_string_prop("Parent page ID", false),
                );
                (
                    "Create Confluence page",
                    props,
                    vec![
                        "space_key".to_string(),
                        "title".to_string(),
                        "content".to_string(),
                    ],
                )
            }
            "confluence_update_page" => {
                let mut props = HashMap::new();
                props.insert(
                    "page_id".to_string(),
                    Self::create_string_prop("Page ID", true),
                );
                props.insert(
                    "title".to_string(),
                    Self::create_string_prop("Page title", true),
                );
                props.insert(
                    "content".to_string(),
                    Self::create_string_prop("Page content in HTML storage format", true),
                );
                props.insert("version_number".to_string(), Self::create_number_prop("Version number (optional). Current version is automatically retrieved and incremented.", 1));
                (
                    "Update Confluence page",
                    props,
                    vec![
                        "page_id".to_string(),
                        "title".to_string(),
                        "content".to_string(),
                    ],
                )
            }
            _ => ("Unknown tool", HashMap::new(), vec![]),
        };

        McpTool {
            name: name.to_string(),
            description: description.to_string(),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties,
                required,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_test_config() -> Config {
        Config {
            atlassian_domain: "test.atlassian.net".to_string(),
            atlassian_email: "test@example.com".to_string(),
            atlassian_api_token: "test-token".to_string(),
            request_timeout_ms: 30000,
            jira_projects_filter: vec![],
            confluence_spaces_filter: vec![],
            jira_search_default_fields: None,
            jira_search_custom_fields: vec![],
            response_exclude_fields: None,
            base_url: "https://test.atlassian.net".to_string(),
        }
    }

    #[tokio::test]
    async fn test_request_handler_creation() {
        let config = Arc::new(create_test_config());
        let handler = RequestHandler::new(config).await;
        assert!(handler.is_ok());
    }

    #[tokio::test]
    async fn test_list_tools_returns_14_tools() {
        let config = Arc::new(create_test_config());
        let handler = RequestHandler::new(config).await.unwrap();
        let tools = handler.list_tools().await;
        assert_eq!(tools.len(), 14);
    }

    #[tokio::test]
    async fn test_list_tools_has_jira_tools() {
        let config = Arc::new(create_test_config());
        let handler = RequestHandler::new(config).await.unwrap();
        let tools = handler.list_tools().await;

        let jira_tools: Vec<_> = tools
            .iter()
            .filter(|t| t.name.starts_with("jira_"))
            .collect();
        assert_eq!(jira_tools.len(), 8);

        // Verify specific Jira tools exist
        assert!(tools.iter().any(|t| t.name == "jira_get_issue"));
        assert!(tools.iter().any(|t| t.name == "jira_search"));
        assert!(tools.iter().any(|t| t.name == "jira_create_issue"));
        assert!(tools.iter().any(|t| t.name == "jira_update_comment"));
    }

    #[tokio::test]
    async fn test_list_tools_has_confluence_tools() {
        let config = Arc::new(create_test_config());
        let handler = RequestHandler::new(config).await.unwrap();
        let tools = handler.list_tools().await;

        let confluence_tools: Vec<_> = tools
            .iter()
            .filter(|t| t.name.starts_with("confluence_"))
            .collect();
        assert_eq!(confluence_tools.len(), 6);

        // Verify specific Confluence tools exist
        assert!(tools.iter().any(|t| t.name == "confluence_search"));
        assert!(tools.iter().any(|t| t.name == "confluence_get_page"));
        assert!(tools.iter().any(|t| t.name == "confluence_create_page"));
    }

    #[tokio::test]
    async fn test_tool_schema_structure() {
        let config = Arc::new(create_test_config());
        let handler = RequestHandler::new(config).await.unwrap();
        let tools = handler.list_tools().await;

        for tool in tools {
            // Every tool must have a name
            assert!(!tool.name.is_empty());

            // Every tool must have a description
            assert!(!tool.description.is_empty());

            // Schema must be "object" type
            assert_eq!(tool.input_schema.schema_type, "object");

            // Must have properties
            assert!(!tool.input_schema.properties.is_empty());

            // Required fields must exist in properties
            for required_field in &tool.input_schema.required {
                assert!(
                    tool.input_schema.properties.contains_key(required_field),
                    "Tool {} missing required property: {}",
                    tool.name,
                    required_field
                );
            }
        }
    }

    #[tokio::test]
    async fn test_jira_search_schema_includes_fields_description() {
        let config = Arc::new(create_test_config());
        let handler = RequestHandler::new(config).await.unwrap();
        let tools = handler.list_tools().await;

        let jira_search = tools.iter().find(|t| t.name == "jira_search").unwrap();

        // Verify jql is required
        assert!(
            jira_search
                .input_schema
                .required
                .contains(&"jql".to_string())
        );

        // Verify fields parameter has description about default fields
        let fields_prop = jira_search.input_schema.properties.get("fields").unwrap();
        assert!(fields_prop.description.is_some());
        let desc = fields_prop.description.as_ref().unwrap();
        assert!(desc.contains("17 default fields")); // Based on DEFAULT_SEARCH_FIELDS count
    }

    #[tokio::test]
    async fn test_jira_get_issue_schema() {
        let config = Arc::new(create_test_config());
        let handler = RequestHandler::new(config).await.unwrap();
        let tools = handler.list_tools().await;

        let tool = tools.iter().find(|t| t.name == "jira_get_issue").unwrap();

        assert_eq!(tool.description, "Get Jira issue by key");
        assert!(
            tool.input_schema
                .required
                .contains(&"issue_key".to_string())
        );
        assert!(tool.input_schema.properties.contains_key("issue_key"));
    }

    #[tokio::test]
    async fn test_confluence_create_page_schema() {
        let config = Arc::new(create_test_config());
        let handler = RequestHandler::new(config).await.unwrap();
        let tools = handler.list_tools().await;

        let tool = tools
            .iter()
            .find(|t| t.name == "confluence_create_page")
            .unwrap();

        assert_eq!(tool.description, "Create Confluence page");
        assert!(
            tool.input_schema
                .required
                .contains(&"space_key".to_string())
        );
        assert!(tool.input_schema.required.contains(&"title".to_string()));
        assert!(tool.input_schema.required.contains(&"content".to_string()));
        assert!(
            !tool
                .input_schema
                .required
                .contains(&"parent_id".to_string())
        ); // Optional
    }

    #[tokio::test]
    async fn test_create_string_prop() {
        let prop = RequestHandler::create_string_prop("Test description", true);
        assert_eq!(prop.property_type, "string");
        assert_eq!(prop.description, Some("Test description".to_string()));
        assert!(prop.default.is_none());
        assert!(prop.enum_values.is_none());
    }

    #[tokio::test]
    async fn test_create_number_prop() {
        let prop = RequestHandler::create_number_prop("Test number", 42);
        assert_eq!(prop.property_type, "number");
        assert_eq!(prop.description, Some("Test number".to_string()));
        assert_eq!(prop.default, Some(json!(42)));
        assert!(prop.enum_values.is_none());
    }
}
