// T022: MCP Protocol Tools List Tests
// Tests the tools/list endpoint functionality

use mcp_atlassian::mcp::types::{JsonRpcRequest, ListToolsResult, Tool, ToolInputSchema};
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_tools_list_request_structure() {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools/list".to_string(),
        params: Some(json!({})),
        id: Some(json!(2)),
    };

    assert_eq!(request.method, "tools/list");
    assert_eq!(request.jsonrpc, "2.0");
    assert_eq!(request.id, Some(json!(2)));
}

#[test]
fn test_list_tools_result_empty() {
    let result = ListToolsResult { tools: vec![] };

    assert_eq!(result.tools.len(), 0);
}

#[test]
fn test_list_tools_result_with_tools() {
    let tools = vec![
        Tool {
            name: "jira_search".to_string(),
            description: "Search Jira issues".to_string(),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: HashMap::new(),
                required: vec!["jql".to_string()],
            },
        },
        Tool {
            name: "confluence_search".to_string(),
            description: "Search Confluence pages".to_string(),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: HashMap::new(),
                required: vec!["query".to_string()],
            },
        },
    ];

    let result = ListToolsResult { tools };

    assert_eq!(result.tools.len(), 2);
    assert_eq!(result.tools[0].name, "jira_search");
    assert_eq!(result.tools[1].name, "confluence_search");
}

#[test]
fn test_tool_input_schema_structure() {
    let mut properties = HashMap::new();
    properties.insert(
        "jql".to_string(),
        mcp_atlassian::mcp::types::Property {
            property_type: "string".to_string(),
            description: Some("JQL query string".to_string()),
            default: None,
            enum_values: None,
        },
    );

    let schema = ToolInputSchema {
        schema_type: "object".to_string(),
        properties,
        required: vec!["jql".to_string()],
    };

    assert_eq!(schema.schema_type, "object");
    assert_eq!(schema.properties.len(), 1);
    assert_eq!(schema.required.len(), 1);
    assert!(schema.properties.contains_key("jql"));
}

#[test]
fn test_tools_list_serialization() {
    let tools = vec![Tool {
        name: "test_tool".to_string(),
        description: "A test tool".to_string(),
        input_schema: ToolInputSchema {
            schema_type: "object".to_string(),
            properties: HashMap::new(),
            required: vec![],
        },
    }];

    let result = ListToolsResult { tools };
    let serialized = serde_json::to_string(&result).unwrap();

    assert!(serialized.contains("test_tool"));
    assert!(serialized.contains("A test tool"));
    assert!(serialized.contains("inputSchema"));
}

#[test]
fn test_tool_with_multiple_required_params() {
    let tool = Tool {
        name: "multi_param_tool".to_string(),
        description: "Tool with multiple required parameters".to_string(),
        input_schema: ToolInputSchema {
            schema_type: "object".to_string(),
            properties: HashMap::new(),
            required: vec![
                "param1".to_string(),
                "param2".to_string(),
                "param3".to_string(),
            ],
        },
    };

    assert_eq!(tool.input_schema.required.len(), 3);
    assert!(tool.input_schema.required.contains(&"param1".to_string()));
    assert!(tool.input_schema.required.contains(&"param2".to_string()));
    assert!(tool.input_schema.required.contains(&"param3".to_string()));
}

#[test]
fn test_tools_list_response_format() {
    let response = json!({
        "jsonrpc": "2.0",
        "result": {
            "tools": [
                {
                    "name": "jira_search",
                    "description": "Search Jira issues",
                    "inputSchema": {
                        "type": "object",
                        "properties": {},
                        "required": ["jql"]
                    }
                }
            ]
        },
        "id": 2
    });

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 2);
    assert!(response["result"]["tools"].is_array());
    assert_eq!(response["result"]["tools"][0]["name"], "jira_search");
}
