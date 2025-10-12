use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// MCP Protocol versions
pub const PROTOCOL_VERSION: &str = "2024-11-05";
pub const PROTOCOL_VERSION_2025: &str = "2025-06-18";

/// JSON-RPC Request
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<Value>,
    pub id: Option<Value>,
}

/// JSON-RPC Response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    pub id: Option<Value>,
}

/// JSON-RPC Error
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// MCP Initialize Request
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InitializeRequest {
    #[serde(alias = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: ClientCapabilities,
    #[serde(alias = "clientInfo")]
    pub client_info: ClientInfo,
}

/// Client Capabilities
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClientCapabilities {
    #[serde(default)]
    pub experimental: HashMap<String, Value>,
}

/// Client Information
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

/// Initialize Result
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InitializeResult {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: ServerCapabilities,
    #[serde(rename = "serverInfo")]
    pub server_info: ServerInfo,
}

/// Server Capabilities
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerCapabilities {
    pub tools: HashMap<String, Value>,
    #[serde(default)]
    pub experimental: HashMap<String, Value>,
}

/// Server Information
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

/// Tool Definition
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: ToolInputSchema,
}

/// Tool Input Schema
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolInputSchema {
    #[serde(rename = "type")]
    pub schema_type: String,
    pub properties: HashMap<String, Property>,
    #[serde(default)]
    pub required: Vec<String>,
}

/// Property Definition
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Property {
    #[serde(rename = "type")]
    pub property_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
    #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<Value>>,
}

/// List Tools Result
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListToolsResult {
    pub tools: Vec<Tool>,
}

/// Call Tool Request
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CallToolRequest {
    pub name: String,
    pub arguments: Value,
}

/// Call Tool Result
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CallToolResult {
    pub content: Vec<ToolContent>,
}

/// Tool Content
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ToolContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { data: String, mime_type: String },
}

/// MCP Error Codes
pub mod error_codes {
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;
}

impl JsonRpcError {
    pub fn parse_error() -> Self {
        Self {
            code: error_codes::PARSE_ERROR,
            message: "Parse error".to_string(),
            data: None,
        }
    }

    pub fn invalid_request() -> Self {
        Self {
            code: error_codes::INVALID_REQUEST,
            message: "Invalid request".to_string(),
            data: None,
        }
    }

    pub fn method_not_found(method: &str) -> Self {
        Self {
            code: error_codes::METHOD_NOT_FOUND,
            message: format!("Method not found: {}", method),
            data: None,
        }
    }

    pub fn invalid_params(message: String) -> Self {
        Self {
            code: error_codes::INVALID_PARAMS,
            message,
            data: None,
        }
    }

    pub fn internal_error(message: String) -> Self {
        Self {
            code: error_codes::INTERNAL_ERROR,
            message,
            data: None,
        }
    }
}

impl JsonRpcResponse {
    pub fn success(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    pub fn error(id: Option<Value>, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(error),
            id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // T020: MCP Protocol types tests

    #[test]
    fn test_protocol_version_constants() {
        assert_eq!(PROTOCOL_VERSION, "2024-11-05");
        assert_eq!(PROTOCOL_VERSION_2025, "2025-06-18");
    }

    #[test]
    fn test_error_codes_constants() {
        assert_eq!(error_codes::PARSE_ERROR, -32700);
        assert_eq!(error_codes::INVALID_REQUEST, -32600);
        assert_eq!(error_codes::METHOD_NOT_FOUND, -32601);
        assert_eq!(error_codes::INVALID_PARAMS, -32602);
        assert_eq!(error_codes::INTERNAL_ERROR, -32603);
    }

    #[test]
    fn test_jsonrpc_error_parse_error() {
        let error = JsonRpcError::parse_error();
        assert_eq!(error.code, error_codes::PARSE_ERROR);
        assert_eq!(error.message, "Parse error");
        assert!(error.data.is_none());
    }

    #[test]
    fn test_jsonrpc_error_invalid_request() {
        let error = JsonRpcError::invalid_request();
        assert_eq!(error.code, error_codes::INVALID_REQUEST);
        assert_eq!(error.message, "Invalid request");
    }

    #[test]
    fn test_jsonrpc_error_method_not_found() {
        let error = JsonRpcError::method_not_found("unknown/method");
        assert_eq!(error.code, error_codes::METHOD_NOT_FOUND);
        assert_eq!(error.message, "Method not found: unknown/method");
    }

    #[test]
    fn test_jsonrpc_error_invalid_params() {
        let error = JsonRpcError::invalid_params("Missing required parameter".to_string());
        assert_eq!(error.code, error_codes::INVALID_PARAMS);
        assert_eq!(error.message, "Missing required parameter");
    }

    #[test]
    fn test_jsonrpc_error_internal_error() {
        let error = JsonRpcError::internal_error("Database connection failed".to_string());
        assert_eq!(error.code, error_codes::INTERNAL_ERROR);
        assert_eq!(error.message, "Database connection failed");
    }

    #[test]
    fn test_jsonrpc_response_success() {
        let result = json!({"status": "ok"});
        let response = JsonRpcResponse::success(Some(json!(1)), result.clone());

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.result, Some(result));
        assert!(response.error.is_none());
        assert_eq!(response.id, Some(json!(1)));
    }

    #[test]
    fn test_jsonrpc_response_error() {
        let error = JsonRpcError::parse_error();
        let response = JsonRpcResponse::error(Some(json!(2)), error.clone());

        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_none());
        assert!(response.error.is_some());
        assert_eq!(response.id, Some(json!(2)));
    }

    #[test]
    fn test_jsonrpc_request_serialization() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/list".to_string(),
            params: Some(json!({})),
            id: Some(json!(1)),
        };

        let serialized = serde_json::to_string(&request).unwrap();
        assert!(serialized.contains("\"jsonrpc\":\"2.0\""));
        assert!(serialized.contains("\"method\":\"tools/list\""));
    }

    #[test]
    fn test_jsonrpc_request_deserialization() {
        let json_str = r#"{"jsonrpc":"2.0","method":"tools/call","params":{"name":"test"},"id":1}"#;
        let request: JsonRpcRequest = serde_json::from_str(json_str).unwrap();

        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.method, "tools/call");
        assert!(request.params.is_some());
        assert_eq!(request.id, Some(json!(1)));
    }

    #[test]
    fn test_initialize_request_structure() {
        let init_req = InitializeRequest {
            protocol_version: PROTOCOL_VERSION_2025.to_string(),
            capabilities: ClientCapabilities {
                experimental: HashMap::new(),
            },
            client_info: ClientInfo {
                name: "test-client".to_string(),
                version: "1.0.0".to_string(),
            },
        };

        assert_eq!(init_req.protocol_version, "2025-06-18");
        assert_eq!(init_req.client_info.name, "test-client");
        assert_eq!(init_req.client_info.version, "1.0.0");
    }

    #[test]
    fn test_initialize_result_structure() {
        let mut tools = HashMap::new();
        tools.insert("listChanged".to_string(), json!(true));

        let init_result = InitializeResult {
            protocol_version: PROTOCOL_VERSION_2025.to_string(),
            capabilities: ServerCapabilities {
                tools,
                experimental: HashMap::new(),
            },
            server_info: ServerInfo {
                name: "mcp-atlassian".to_string(),
                version: "0.1.0".to_string(),
            },
        };

        assert_eq!(init_result.protocol_version, "2025-06-18");
        assert_eq!(init_result.server_info.name, "mcp-atlassian");
    }

    #[test]
    fn test_tool_structure() {
        let mut properties = HashMap::new();
        properties.insert(
            "query".to_string(),
            Property {
                property_type: "string".to_string(),
                description: Some("Search query".to_string()),
                default: None,
                enum_values: None,
            },
        );

        let tool = Tool {
            name: "jira_search".to_string(),
            description: "Search Jira issues".to_string(),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties,
                required: vec!["query".to_string()],
            },
        };

        assert_eq!(tool.name, "jira_search");
        assert_eq!(tool.input_schema.schema_type, "object");
        assert_eq!(tool.input_schema.required.len(), 1);
    }

    #[test]
    fn test_call_tool_request() {
        let request = CallToolRequest {
            name: "jira_search".to_string(),
            arguments: json!({"jql": "status = Open"}),
        };

        assert_eq!(request.name, "jira_search");
        assert_eq!(request.arguments["jql"], "status = Open");
    }

    #[test]
    fn test_tool_content_text() {
        let content = ToolContent::Text {
            text: "Test result".to_string(),
        };

        match content {
            ToolContent::Text { text } => assert_eq!(text, "Test result"),
            _ => panic!("Expected Text variant"),
        }
    }

    #[test]
    fn test_tool_content_image() {
        let content = ToolContent::Image {
            data: "base64data".to_string(),
            mime_type: "image/png".to_string(),
        };

        match content {
            ToolContent::Image { data, mime_type } => {
                assert_eq!(data, "base64data");
                assert_eq!(mime_type, "image/png");
            }
            _ => panic!("Expected Image variant"),
        }
    }

    #[test]
    fn test_call_tool_result_structure() {
        let result = CallToolResult {
            content: vec![
                ToolContent::Text {
                    text: "Result 1".to_string(),
                },
                ToolContent::Text {
                    text: "Result 2".to_string(),
                },
            ],
        };

        assert_eq!(result.content.len(), 2);
    }

    #[test]
    fn test_property_with_enum() {
        let property = Property {
            property_type: "string".to_string(),
            description: Some("Status field".to_string()),
            default: None,
            enum_values: Some(vec![json!("Open"), json!("In Progress"), json!("Closed")]),
        };

        assert_eq!(property.property_type, "string");
        assert_eq!(property.enum_values.as_ref().unwrap().len(), 3);
    }
}
