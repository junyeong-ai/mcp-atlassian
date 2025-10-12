use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::config::Config;

use super::handlers::RequestHandler;
use super::types::*;

pub struct McpServer {
    config: Arc<Config>,
    handler: Arc<RequestHandler>,
    initialized: Arc<RwLock<bool>>,
}

impl McpServer {
    pub async fn new(config: Config) -> Result<Self> {
        let config = Arc::new(config);
        let handler = RequestHandler::new(config.clone()).await?;

        Ok(Self {
            config,
            handler: Arc::new(handler),
            initialized: Arc::new(RwLock::new(false)),
        })
    }

    pub async fn run(&self) -> Result<()> {
        info!("Starting MCP server for Atlassian");

        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();
        let mut reader = BufReader::new(stdin);
        let mut stdout = stdout;

        let mut buffer = String::new();
        let mut empty_reads = 0;

        loop {
            buffer.clear();

            // Read a line from stdin
            match reader.read_line(&mut buffer).await {
                Ok(0) => {
                    empty_reads += 1;

                    // Give it a few chances before exiting
                    if empty_reads > 3 {
                        info!("Client disconnected (EOF)");
                        break;
                    }
                    // Small delay before retrying
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    continue;
                }
                Ok(_) => {
                    empty_reads = 0; // Reset counter on successful read
                    let trimmed = buffer.trim();
                    if trimmed.is_empty() {
                        continue;
                    }

                    debug!("Received: {}", trimmed);

                    // Process the request
                    match self.process_request(trimmed).await {
                        Ok(Some(response)) => {
                            let response_str = serde_json::to_string(&response)?;
                            debug!("Sending response: {}", response_str);

                            stdout.write_all(response_str.as_bytes()).await?;
                            stdout.write_all(b"\n").await?;
                            stdout.flush().await?;
                        }
                        Ok(None) => {
                            debug!("Notification received, no response sent");
                        }
                        Err(e) => {
                            error!("Error processing request: {}", e);

                            // Send error response
                            let error_response = JsonRpcResponse::error(
                                None,
                                JsonRpcError::internal_error(e.to_string()),
                            );

                            let response_str = serde_json::to_string(&error_response)?;
                            stdout.write_all(response_str.as_bytes()).await?;
                            stdout.write_all(b"\n").await?;
                            stdout.flush().await?;
                        }
                    }
                }
                Err(e) => {
                    error!("Error reading from stdin: {}", e);
                    break;
                }
            }
        }

        info!("MCP server shutting down");
        Ok(())
    }

    async fn process_request(&self, input: &str) -> Result<Option<JsonRpcResponse>> {
        // Parse JSON-RPC request
        let request: JsonRpcRequest = match serde_json::from_str(input) {
            Ok(req) => req,
            Err(e) => {
                warn!("Failed to parse request: {}", e);
                return Ok(Some(JsonRpcResponse::error(
                    None,
                    JsonRpcError::parse_error(),
                )));
            }
        };

        // Validate JSON-RPC version
        if request.jsonrpc != "2.0" {
            return Ok(Some(JsonRpcResponse::error(
                request.id.clone(),
                JsonRpcError::invalid_request(),
            )));
        }

        // Route to appropriate handler
        match request.method.as_str() {
            "initialize" => self.handle_initialize(request).await.map(Some),
            "initialized" | "notifications/initialized" => self.handle_initialized(request).await,
            "tools/list" => self.handle_list_tools(request).await.map(Some),
            "tools/call" => self.handle_call_tool(request).await.map(Some),
            "prompts/list" => self.handle_list_prompts(request).await.map(Some),
            "resources/list" => self.handle_list_resources(request).await.map(Some),
            _ => {
                warn!("Unknown method: {}", request.method);
                Ok(Some(JsonRpcResponse::error(
                    request.id,
                    JsonRpcError::method_not_found(&request.method),
                )))
            }
        }
    }

    async fn handle_initialize(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        debug!("Handling initialize request");

        // Parse initialize params (optional for flexibility)
        let protocol_version = if let Some(params) = request.params {
            if let Ok(init_req) = serde_json::from_value::<InitializeRequest>(params) {
                // Support both protocol versions
                if init_req.protocol_version.starts_with("2025") {
                    PROTOCOL_VERSION_2025.to_string()
                } else {
                    PROTOCOL_VERSION.to_string()
                }
            } else {
                PROTOCOL_VERSION_2025.to_string()
            }
        } else {
            PROTOCOL_VERSION_2025.to_string()
        };

        // Create initialize result with empty tools object (like mcp-slack)
        let result = InitializeResult {
            protocol_version,
            capabilities: ServerCapabilities {
                tools: HashMap::new(), // Empty tools object
                experimental: HashMap::new(),
            },
            server_info: ServerInfo {
                name: "mcp-atlassian".to_string(),
                version: "0.1.0".to_string(),
            },
        };

        Ok(JsonRpcResponse::success(
            request.id,
            serde_json::to_value(result)?,
        ))
    }

    async fn handle_initialized(&self, request: JsonRpcRequest) -> Result<Option<JsonRpcResponse>> {
        debug!("Handling initialized notification");

        let mut initialized = self.initialized.write().await;
        *initialized = true;

        // Notifications don't get responses
        if request.id.is_none() {
            Ok(None)
        } else {
            // If it has an ID, it's a request and needs a response
            Ok(Some(JsonRpcResponse::success(request.id, Value::Null)))
        }
    }

    async fn handle_list_tools(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        debug!("Handling tools/list request");

        // Check if initialized
        let initialized = self.initialized.read().await;
        if !*initialized {
            return Ok(JsonRpcResponse::error(
                request.id,
                JsonRpcError::internal_error("Server not initialized".to_string()),
            ));
        }

        let tools = self.handler.list_tools().await;
        let result = ListToolsResult { tools };

        Ok(JsonRpcResponse::success(
            request.id,
            serde_json::to_value(result)?,
        ))
    }

    async fn handle_call_tool(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        debug!("Handling tools/call request");

        // Check if initialized
        let initialized = self.initialized.read().await;
        if !*initialized {
            return Ok(JsonRpcResponse::error(
                request.id,
                JsonRpcError::internal_error("Server not initialized".to_string()),
            ));
        }

        // Parse call tool params
        let params: CallToolRequest = match request.params {
            Some(p) => serde_json::from_value(p)?,
            None => {
                return Ok(JsonRpcResponse::error(
                    request.id,
                    JsonRpcError::invalid_params("Missing params".to_string()),
                ));
            }
        };

        debug!("Executing tool: {}", params.name);

        // Execute tool
        match self
            .handler
            .call_tool(&params.name, params.arguments, &self.config)
            .await
        {
            Ok(result) => Ok(JsonRpcResponse::success(
                request.id,
                serde_json::to_value(result)?,
            )),
            Err(e) => {
                error!("Tool execution failed: {}", e);
                Ok(JsonRpcResponse::error(
                    request.id,
                    JsonRpcError::internal_error(e.to_string()),
                ))
            }
        }
    }

    async fn handle_list_prompts(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        debug!("Handling prompts/list request");

        // We don't have prompts, return empty list
        let result = serde_json::json!({
            "prompts": []
        });

        Ok(JsonRpcResponse::success(request.id, result))
    }

    async fn handle_list_resources(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        debug!("Handling resources/list request");

        // We don't have resources, return empty list
        let result = serde_json::json!({
            "resources": []
        });

        Ok(JsonRpcResponse::success(request.id, result))
    }
}
