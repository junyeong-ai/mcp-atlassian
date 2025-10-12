// T021: MCP Protocol Initialize Tests
// Tests the MCP protocol initialization flow

use mcp_atlassian::mcp::types::{
    ClientCapabilities, ClientInfo, InitializeRequest, JsonRpcRequest,
    PROTOCOL_VERSION, PROTOCOL_VERSION_2025,
};
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_initialize_request_with_2024_version() {
    let init_request = InitializeRequest {
        protocol_version: PROTOCOL_VERSION.to_string(),
        capabilities: ClientCapabilities {
            experimental: HashMap::new(),
        },
        client_info: ClientInfo {
            name: "test-client".to_string(),
            version: "1.0.0".to_string(),
        },
    };

    assert_eq!(init_request.protocol_version, "2024-11-05");
    assert_eq!(init_request.client_info.name, "test-client");
    assert_eq!(init_request.client_info.version, "1.0.0");
}

#[test]
fn test_initialize_request_with_2025_version() {
    let init_request = InitializeRequest {
        protocol_version: PROTOCOL_VERSION_2025.to_string(),
        capabilities: ClientCapabilities {
            experimental: HashMap::new(),
        },
        client_info: ClientInfo {
            name: "test-client".to_string(),
            version: "2.0.0".to_string(),
        },
    };

    assert_eq!(init_request.protocol_version, "2025-06-18");
    assert_eq!(init_request.client_info.name, "test-client");
}

#[test]
fn test_initialize_request_serialization() {
    let init_request = InitializeRequest {
        protocol_version: PROTOCOL_VERSION_2025.to_string(),
        capabilities: ClientCapabilities {
            experimental: HashMap::new(),
        },
        client_info: ClientInfo {
            name: "test-client".to_string(),
            version: "1.0.0".to_string(),
        },
    };

    let serialized = serde_json::to_string(&init_request).unwrap();
    // Note: serde may serialize as either protocol_version or protocolVersion depending on aliases
    assert!(serialized.contains("protocol") || serialized.contains("Protocol"));
    assert!(serialized.contains("2025-06-18"));
    assert!(serialized.contains("client") || serialized.contains("Client"));
}

#[test]
fn test_initialize_request_deserialization() {
    let json_str = r#"{
        "protocolVersion": "2025-06-18",
        "capabilities": {
            "experimental": {}
        },
        "clientInfo": {
            "name": "test-client",
            "version": "1.0.0"
        }
    }"#;

    let request: InitializeRequest = serde_json::from_str(json_str).unwrap();
    assert_eq!(request.protocol_version, "2025-06-18");
    assert_eq!(request.client_info.name, "test-client");
}

#[test]
fn test_initialize_jsonrpc_request_structure() {
    let params = json!({
        "protocolVersion": "2025-06-18",
        "capabilities": {},
        "clientInfo": {
            "name": "test-client",
            "version": "1.0.0"
        }
    });

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "initialize".to_string(),
        params: Some(params),
        id: Some(json!(1)),
    };

    assert_eq!(request.method, "initialize");
    assert!(request.params.is_some());
    assert_eq!(request.id, Some(json!(1)));
}

#[test]
fn test_initialize_with_experimental_capabilities() {
    let mut experimental = HashMap::new();
    experimental.insert("customFeature".to_string(), json!(true));

    let init_request = InitializeRequest {
        protocol_version: PROTOCOL_VERSION_2025.to_string(),
        capabilities: ClientCapabilities { experimental },
        client_info: ClientInfo {
            name: "test-client".to_string(),
            version: "1.0.0".to_string(),
        },
    };

    assert_eq!(init_request.capabilities.experimental.len(), 1);
    assert_eq!(
        init_request.capabilities.experimental.get("customFeature"),
        Some(&json!(true))
    );
}
