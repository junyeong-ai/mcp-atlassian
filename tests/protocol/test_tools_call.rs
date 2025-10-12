// T023: MCP Protocol Tools Call Tests
// Tests the tools/call endpoint functionality

use mcp_atlassian::mcp::types::{CallToolRequest, CallToolResult, JsonRpcRequest, ToolContent};
use serde_json::json;

#[test]
fn test_tools_call_request_structure() {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "jira_search",
            "arguments": {
                "jql": "status = Open"
            }
        })),
        id: Some(json!(3)),
    };

    assert_eq!(request.method, "tools/call");
    assert_eq!(request.jsonrpc, "2.0");
    assert!(request.params.is_some());
}

#[test]
fn test_call_tool_request_jira_search() {
    let request = CallToolRequest {
        name: "jira_search".to_string(),
        arguments: json!({
            "jql": "project = TEST AND status = Open",
            "limit": 20
        }),
    };

    assert_eq!(request.name, "jira_search");
    assert_eq!(request.arguments["jql"], "project = TEST AND status = Open");
    assert_eq!(request.arguments["limit"], 20);
}

#[test]
fn test_call_tool_request_confluence_search() {
    let request = CallToolRequest {
        name: "confluence_search".to_string(),
        arguments: json!({
            "query": "type=page",
            "limit": 10
        }),
    };

    assert_eq!(request.name, "confluence_search");
    assert_eq!(request.arguments["query"], "type=page");
}

#[test]
fn test_call_tool_result_with_text_content() {
    let result = CallToolResult {
        content: vec![ToolContent::Text {
            text: "Search completed successfully".to_string(),
        }],
    };

    assert_eq!(result.content.len(), 1);
    match &result.content[0] {
        ToolContent::Text { text } => {
            assert_eq!(text, "Search completed successfully");
        }
        _ => panic!("Expected Text content"),
    }
}

#[test]
fn test_call_tool_result_with_multiple_content() {
    let result = CallToolResult {
        content: vec![
            ToolContent::Text {
                text: "Result 1".to_string(),
            },
            ToolContent::Text {
                text: "Result 2".to_string(),
            },
            ToolContent::Text {
                text: "Result 3".to_string(),
            },
        ],
    };

    assert_eq!(result.content.len(), 3);
}

#[test]
fn test_call_tool_result_serialization() {
    let result = CallToolResult {
        content: vec![ToolContent::Text {
            text: "Test output".to_string(),
        }],
    };

    let serialized = serde_json::to_string(&result).unwrap();
    assert!(serialized.contains("Test output"));
    assert!(serialized.contains("\"type\":\"text\""));
}

#[test]
fn test_call_tool_request_serialization() {
    let request = CallToolRequest {
        name: "jira_get_issue".to_string(),
        arguments: json!({"issue_key": "PROJ-123"}),
    };

    let serialized = serde_json::to_string(&request).unwrap();
    assert!(serialized.contains("jira_get_issue"));
    assert!(serialized.contains("PROJ-123"));
}

#[test]
fn test_call_tool_request_deserialization() {
    let json_str = r#"{
        "name": "jira_create_issue",
        "arguments": {
            "project_key": "TEST",
            "summary": "New issue",
            "issue_type": "Task"
        }
    }"#;

    let request: CallToolRequest = serde_json::from_str(json_str).unwrap();
    assert_eq!(request.name, "jira_create_issue");
    assert_eq!(request.arguments["project_key"], "TEST");
    assert_eq!(request.arguments["summary"], "New issue");
}

#[test]
fn test_tools_call_response_format() {
    let response = json!({
        "jsonrpc": "2.0",
        "result": {
            "content": [
                {
                    "type": "text",
                    "text": "Operation completed"
                }
            ]
        },
        "id": 3
    });

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 3);
    assert!(response["result"]["content"].is_array());
    assert_eq!(response["result"]["content"][0]["type"], "text");
}

#[test]
fn test_call_tool_with_complex_arguments() {
    let request = CallToolRequest {
        name: "jira_update_issue".to_string(),
        arguments: json!({
            "issue_key": "PROJ-123",
            "fields": {
                "summary": "Updated summary",
                "priority": {"name": "High"},
                "labels": ["urgent", "bug"]
            }
        }),
    };

    assert_eq!(request.name, "jira_update_issue");
    assert_eq!(request.arguments["issue_key"], "PROJ-123");
    assert_eq!(request.arguments["fields"]["summary"], "Updated summary");
    assert_eq!(request.arguments["fields"]["priority"]["name"], "High");
}
