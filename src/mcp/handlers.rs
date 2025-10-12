use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use serde_json::Value;

use crate::config::Config;
use crate::tools::{jira, confluence};
use crate::tools::ToolHandler;

use super::types::{Tool as McpTool, ToolInputSchema, Property, CallToolResult, ToolContent};

pub struct RequestHandler {
    tools: HashMap<String, Arc<dyn ToolHandler>>,
}

impl RequestHandler {
    pub async fn new(_config: Arc<Config>) -> Result<Self> {
        let mut tools: HashMap<String, Arc<dyn ToolHandler>> = HashMap::new();

        // Register Jira tools
        tools.insert("jira_get_issue".to_string(), Arc::new(jira::GetIssueHandler));
        tools.insert("jira_search".to_string(), Arc::new(jira::SearchHandler));
        tools.insert("jira_create_issue".to_string(), Arc::new(jira::CreateIssueHandler));
        tools.insert("jira_update_issue".to_string(), Arc::new(jira::UpdateIssueHandler));
        tools.insert("jira_add_comment".to_string(), Arc::new(jira::AddCommentHandler));
        tools.insert("jira_transition_issue".to_string(), Arc::new(jira::TransitionIssueHandler));
        tools.insert("jira_get_transitions".to_string(), Arc::new(jira::GetTransitionsHandler));

        // Register Confluence tools
        tools.insert("confluence_search".to_string(), Arc::new(confluence::SearchHandler));
        tools.insert("confluence_get_page".to_string(), Arc::new(confluence::GetPageHandler));
        tools.insert("confluence_get_page_children".to_string(), Arc::new(confluence::GetPageChildrenHandler));
        tools.insert("confluence_get_comments".to_string(), Arc::new(confluence::GetCommentsHandler));
        tools.insert("confluence_create_page".to_string(), Arc::new(confluence::CreatePageHandler));
        tools.insert("confluence_update_page".to_string(), Arc::new(confluence::UpdatePageHandler));

        Ok(Self { tools })
    }

    pub async fn list_tools(&self) -> Vec<McpTool> {
        let mut tool_list = Vec::new();

        for name in self.tools.keys() {
            tool_list.push(self.tool_to_mcp_tool(name));
        }

        tool_list
    }

    pub async fn call_tool(&self, name: &str, arguments: Value, config: &Config) -> Result<CallToolResult> {
        let tool = self.tools.get(name)
            .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", name))?;

        let result = tool.execute(arguments, config).await?;

        // Convert result to tool content
        let content = if let Some(text) = result.as_str() {
            vec![ToolContent::Text { text: text.to_string() }]
        } else {
            vec![ToolContent::Text { text: serde_json::to_string_pretty(&result)? }]
        };

        Ok(CallToolResult { content })
    }

    fn create_string_prop(description: &str, _required: bool) -> Property {
        Property {
            property_type: "string".to_string(),
            description: Some(description.to_string()),
            default: None,
            enum_values: None,
        }
    }

    fn create_number_prop(description: &str, default: i32) -> Property {
        Property {
            property_type: "number".to_string(),
            description: Some(description.to_string()),
            default: Some(Value::Number(default.into())),
            enum_values: None,
        }
    }

    fn tool_to_mcp_tool(&self, name: &str) -> McpTool {
        let (description, properties, required) = match name {
            // Jira tools
            "jira_get_issue" => {
                let mut props = HashMap::new();
                props.insert("issue_key".to_string(), Self::create_string_prop("Issue key (e.g., 'PROJECT-123'). Case-sensitive.", true));
                ("Get Jira issue by key", props, vec!["issue_key".to_string()])
            }
            "jira_search" => {
                let mut props = HashMap::new();
                props.insert("jql".to_string(), Self::create_string_prop("JQL query. Must include search condition before ORDER BY (e.g., 'project = KEY ORDER BY created DESC'). ORDER BY only works with orderable fields (dates, versions).", true));
                props.insert("limit".to_string(), Self::create_number_prop("Max results", 10));
                ("Search Jira issues using JQL", props, vec!["jql".to_string()])
            }
            "jira_create_issue" => {
                let mut props = HashMap::new();
                props.insert("project_key".to_string(), Self::create_string_prop("Project key", true));
                props.insert("summary".to_string(), Self::create_string_prop("Issue summary", true));
                props.insert("issue_type".to_string(), Self::create_string_prop("Issue type name (e.g., 'Task', 'Bug', 'Story').", true));
                props.insert("description".to_string(), Self::create_string_prop("Issue description", false));
                ("Create Jira issue", props, vec!["project_key".to_string(), "summary".to_string(), "issue_type".to_string()])
            }
            "jira_update_issue" => {
                let mut props = HashMap::new();
                props.insert("issue_key".to_string(), Self::create_string_prop("Issue key", true));
                props.insert("fields".to_string(), Property {
                    property_type: "object".to_string(),
                    description: Some("Fields to update as JSON object (e.g., {\"summary\": \"New title\"}). Custom fields use 'customfield_*' format.".to_string()),
                    default: None,
                    enum_values: None,
                });
                ("Update Jira issue", props, vec!["issue_key".to_string(), "fields".to_string()])
            }
            "jira_add_comment" => {
                let mut props = HashMap::new();
                props.insert("issue_key".to_string(), Self::create_string_prop("Issue key", true));
                props.insert("comment".to_string(), Self::create_string_prop("Comment text (plain text). Automatically converted to Atlassian Document Format.", true));
                ("Add comment to Jira issue", props, vec!["issue_key".to_string(), "comment".to_string()])
            }
            "jira_transition_issue" => {
                let mut props = HashMap::new();
                props.insert("issue_key".to_string(), Self::create_string_prop("Issue key", true));
                props.insert("transition_id".to_string(), Self::create_string_prop("Transition ID. Get available transition IDs using jira_get_transitions for the issue's current status.", true));
                ("Transition Jira issue status", props, vec!["issue_key".to_string(), "transition_id".to_string()])
            }
            "jira_get_transitions" => {
                let mut props = HashMap::new();
                props.insert("issue_key".to_string(), Self::create_string_prop("Issue key", true));
                ("Get Jira issue transitions", props, vec!["issue_key".to_string()])
            }
            // Confluence tools
            "confluence_search" => {
                let mut props = HashMap::new();
                props.insert("query".to_string(), Self::create_string_prop("CQL query. Format: field operator value (e.g., 'type=page AND space=\"SPACE\"'). Use text ~ \"keyword\" for text search.", true));
                props.insert("limit".to_string(), Self::create_number_prop("Max results", 10));
                ("Search Confluence using CQL", props, vec!["query".to_string()])
            }
            "confluence_get_page" => {
                let mut props = HashMap::new();
                props.insert("page_id".to_string(), Self::create_string_prop("Page ID", true));
                ("Get Confluence page by ID", props, vec!["page_id".to_string()])
            }
            "confluence_get_page_children" => {
                let mut props = HashMap::new();
                props.insert("page_id".to_string(), Self::create_string_prop("Page ID", true));
                ("Get page child pages", props, vec!["page_id".to_string()])
            }
            "confluence_get_comments" => {
                let mut props = HashMap::new();
                props.insert("page_id".to_string(), Self::create_string_prop("Page ID", true));
                ("Get page comments", props, vec!["page_id".to_string()])
            }
            "confluence_create_page" => {
                let mut props = HashMap::new();
                props.insert("space_key".to_string(), Self::create_string_prop("Space key", true));
                props.insert("title".to_string(), Self::create_string_prop("Page title", true));
                props.insert("content".to_string(), Self::create_string_prop("Page content in HTML storage format.", true));
                props.insert("parent_id".to_string(), Self::create_string_prop("Parent page ID", false));
                ("Create Confluence page", props, vec!["space_key".to_string(), "title".to_string(), "content".to_string()])
            }
            "confluence_update_page" => {
                let mut props = HashMap::new();
                props.insert("page_id".to_string(), Self::create_string_prop("Page ID", true));
                props.insert("title".to_string(), Self::create_string_prop("Page title", true));
                props.insert("content".to_string(), Self::create_string_prop("Page content", true));
                props.insert("version_number".to_string(), Self::create_number_prop("Version number (optional). Current version is automatically retrieved and incremented.", 1));
                ("Update Confluence page", props, vec!["page_id".to_string(), "title".to_string(), "content".to_string()])
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