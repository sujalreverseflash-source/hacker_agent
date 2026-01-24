use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};

mod api;
mod services;
mod tools;
mod prompts;

/// Basic JSON-RPC-like request type.
#[derive(Debug, Deserialize)]
struct RpcRequest {
    #[serde(default)]
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}
//Value is generic typed JSON value that can hold any valid JSON structure
/// Basic JSON-RPC-like response type.
#[derive(Debug, Serialize)]
struct RpcResponse {
    jsonrpc: &'static str,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<RpcError>,
}
//Optional fields are skipped if not present as part of the RPC response

#[derive(Debug, Serialize)]
struct RpcError {
    code: i32,
    message: String,
}

/// Generic tool trait, similar in spirit to a fastmcp tool.
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;

    /// JSON Schema for this tool's `input` parameter (MCP `inputSchema`).
    /// By default, accept any JSON object. Individual tools can override.
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "description": "Arbitrary JSON object"
        })
    }

    async fn execute(&self, input: Value) -> Result<Value>;
}

/// Registry of tools that can be listed and called.
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register<T: Tool + 'static>(&mut self, tool: T) {
        self.tools
            .insert(tool.name().to_string(), Arc::new(tool));
    }

    fn list(&self) -> Vec<Value> {
        self.tools
            .values()
            .map(|t| {
                json!({
                    "name": t.name(),
                    "description": t.description(),
                    "inputSchema": t.input_schema(),
                })
            })
            .collect()
    }

    async fn call(&self, name: &str, input: Value) -> Result<Value> {
        let tool = self
            .tools
            .get(name)
            .ok_or_else(|| anyhow::anyhow!(format!("Unknown tool: {name}")))?;
        tool.execute(input).await
    }
}

/// Parameters for tools.call.
#[derive(Debug, Deserialize)]
struct ToolCallParams {
    name: String,
    #[serde(default)]
    input: Value,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Build the tool registry.
    let mut reg = ToolRegistry::new();
    tools::register_all_tools(&mut reg);
    let registry = Arc::new(reg);

    // 2. Set up stdin/stdout JSON loop.
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut reader = BufReader::new(stdin).lines();
    let mut writer = io::BufWriter::new(stdout);

    while let Some(line) = reader.next_line().await? {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Try to parse a request.
        let req: RpcRequest = match serde_json::from_str(line) {
            Ok(r) => r,
            Err(_err) => {
                // If we can't parse the incoming JSON at all, just ignore it.
                // MCP clients can send various notifications; we don't want to
                // emit malformed error responses that confuse the client.
                continue;
            }
        };

        // Notifications in MCP/JSON-RPC do not include an `id` and must not
        // receive a response. Only handle messages with an ID as requests.
        let Some(id) = req.id.clone() else {
            continue;
        };

        // Handle the request and send a response.
        let resp = handle_request(registry.clone(), id, req).await;
        let text = serde_json::to_string(&resp)?;
        writer.write_all(text.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;
    }

    Ok(())
}

/// Dispatches methods like `tools/list` and `tools/call`.
async fn handle_request(registry: Arc<ToolRegistry>, id: Value, req: RpcRequest) -> RpcResponse {
    match req.method.as_str() {
        // MCP / JSON-RPC 2.0 initialization handshake.
        // Cursor (and other MCP clients) will generally send an `initialize`
        // request before calling any tools. We respond with minimal
        // capabilities so the client treats the server as valid.
        "initialize" => {
            // MCP expects the result object to include a `protocolVersion`
            // string. We try to echo back whatever the client sent; if it's
            // missing, we fall back to a reasonable default.
            let protocol_version = req
                .params
                .get("protocolVersion")
                .and_then(|v| v.as_str())
                .unwrap_or("2024-11-05");

            ok(
                id,
                json!({
                    "protocolVersion": protocol_version,
                    "capabilities": {
                        "tools": {
                            "listChanged": true
                        },
                        "prompts": {
                            "listChanged": true
                        }
                    },
                    "serverInfo": {
                        "name": "hacker_agent",
                        "version": "0.1.0"
                    }
                }),
            )
        }
        "tools/list" => {
            let tools = registry.list();
            ok(id, json!({ "tools": tools }))
        }
        "tools/call" => {
            let parsed: Result<ToolCallParams, _> = serde_json::from_value(req.params);
            let params = match parsed {
                Ok(p) => p,
                Err(err) => {
                    return err_resp(id, -32602, format!("Invalid params: {err}"));
                }
            };

            match registry.call(&params.name, params.input).await {
                Ok(value) => ok(id, json!({ "output": value })),
                Err(err) => err_resp(id, -32000, format!("Tool error: {err}")),
            }
        }
        "prompts/list" => {
            let prompts = prompts::list_prompts();
            ok(id, json!({ "prompts": prompts }))
        }
        "prompts/get" => {
            let parsed: Result<prompts::PromptGetParams, _> = serde_json::from_value(req.params);
            let params = match parsed {
                Ok(p) => p,
                Err(err) => {
                    return err_resp(id, -32602, format!("Invalid params: {err}"));
                }
            };

            match prompts::get_prompt(&params.name, params.arguments) {
                Ok(prompt) => ok(id, json!({ "prompt": prompt })),
                Err(err) => err_resp(id, -32601, format!("Prompt not found: {err}")),
            }
        }
        _ => err_resp(
            id,
            -32601,
            format!("Method not found: {}", req.method),
        ),
    }
}

fn ok(id: Value, result: Value) -> RpcResponse {
    RpcResponse {
        jsonrpc: "2.0",
        id,
        result: Some(result),
        error: None,
    }
}

fn err_resp(id: Value, code: i32, message: String) -> RpcResponse {
    RpcResponse {
        jsonrpc: "2.0",
        id,
        result: None,
        error: Some(RpcError { code, message }),
    }
}
