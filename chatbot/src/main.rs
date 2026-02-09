use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use axum::{
    extract::State,
    body::Bytes,
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use axum::response::sse::{Event, Sse};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::mpsc;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio_stream::wrappers::ReceiverStream;

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

    async fn execute(&self, input: Value, ctx: ExecutionContext) -> Result<Value>;
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

    async fn call_with_ctx(&self, name: &str, input: Value, ctx: ExecutionContext) -> Result<Value> {
        let tool = self
            .tools
            .get(name)
            .ok_or_else(|| anyhow::anyhow!(format!("Unknown tool: {name}")))?;
        tool.execute(input, ctx).await
    }
}

/// Parameters for tools.call.
#[derive(Debug, Deserialize)]
struct ToolCallParams {
    name: String,
    #[serde(default, alias = "arguments")]
    input: Value,
    #[serde(default, rename = "_meta")]
    meta: Value,
}

#[async_trait]
pub trait Notifier: Send + Sync {
    async fn send(&self, msg: Value);
}

#[derive(Clone)]
pub struct ChannelNotifier {
    tx: mpsc::Sender<Value>,
}

#[async_trait]
impl Notifier for ChannelNotifier {
    async fn send(&self, msg: Value) {
        // Best-effort; if receiver is dropped we just stop notifying.
        let _ = self.tx.send(msg).await;
    }
}

#[derive(Clone)]
pub struct ExecutionContext {
    progress_token: Option<Value>,
    notifier: Option<Arc<dyn Notifier>>,
}

impl ExecutionContext {
    pub fn noop() -> Self {
        Self {
            progress_token: None,
            notifier: None,
        }
    }

    pub fn new(_request_id: Value, meta: Value, notifier: Arc<dyn Notifier>) -> Self {
        let progress_token = meta
            .get("progressToken")
            .cloned()
            .or_else(|| meta.get("progress_token").cloned());
        Self {
            progress_token,
            notifier: Some(notifier),
        }
    }

    pub fn progress_token(&self) -> Option<&Value> {
        self.progress_token.as_ref()
    }

    pub async fn notify_progress(&self, progress: f64, total: Option<f64>, message: Option<String>) {
        let (Some(token), Some(notifier)) = (self.progress_token.clone(), self.notifier.clone()) else {
            return;
        };

        let mut params = json!({
            "progressToken": token,
            "progress": progress
        });
        if let Some(t) = total {
            params["total"] = json!(t);
        }
        if let Some(m) = message {
            params["message"] = json!(m);
        }

        notifier
            .send(json!({
                "jsonrpc": "2.0",
                "method": "notifications/progress",
                "params": params
            }))
            .await;
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Build the tool registry.
    let mut reg = ToolRegistry::new();
    tools::register_all_tools(&mut reg);
    let registry = Arc::new(reg);

    let args: Vec<String> = std::env::args().collect();
    let transport = std::env::var("MCP_TRANSPORT").unwrap_or_else(|_| "stdio".to_string());
    let use_http = args.iter().any(|a| a == "--http") || transport == "http" || transport == "sse";

    if use_http {
        run_streamable_http(registry).await
    } else {
        run_stdio(registry).await
    }
}

async fn run_stdio(registry: Arc<ToolRegistry>) -> Result<()> {
    // Writer task for responses + notifications.
    let (tx, mut rx) = mpsc::channel::<Value>(1024);
    let stdout = io::stdout();
    let mut writer = io::BufWriter::new(stdout);
    let writer_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Ok(text) = serde_json::to_string(&msg) {
                let _ = writer.write_all(text.as_bytes()).await;
                let _ = writer.write_all(b"\n").await;
                let _ = writer.flush().await;
            }
        }
    });

    let notifier: Arc<dyn Notifier> = Arc::new(ChannelNotifier { tx: tx.clone() });

    // stdin loop
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin).lines();
    while let Some(line) = reader.next_line().await? {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let req: RpcRequest = match serde_json::from_str(line) {
            Ok(r) => r,
            Err(_) => continue,
        };

        let Some(id) = req.id.clone() else {
            // notifications: ignore for now
            continue;
        };

        let resp = handle_request(registry.clone(), id.clone(), req, notifier.clone()).await;
        let _ = tx.send(serde_json::to_value(resp)?).await;
    }

    drop(tx);
    let _ = writer_task.await;
    Ok(())
}

#[derive(Clone)]
struct HttpState {
    registry: Arc<ToolRegistry>,
}

async fn run_streamable_http(registry: Arc<ToolRegistry>) -> Result<()> {
    let port: u16 = std::env::var("MCP_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3000);

    let host = std::env::var("MCP_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let addr: SocketAddr = format!("{host}:{port}").parse()?;

    let app = Router::new()
        .route("/mcp", post(mcp_post).get(mcp_get))
        .with_state(HttpState { registry });

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

fn validate_origin(headers: &HeaderMap) -> bool {
    // Spec requires validating Origin to mitigate DNS rebinding.
    // For local dev, we allow:
    // - missing Origin
    // - localhost / 127.0.0.1 origins
    let Some(origin) = headers.get("origin").and_then(|v| v.to_str().ok()) else {
        return true;
    };
    let origin = origin.to_ascii_lowercase();
    origin.contains("://localhost") || origin.contains("://127.0.0.1")
}

async fn mcp_get(State(_state): State<HttpState>) -> impl IntoResponse {
    // Optional in spec; we don't expose a general-purpose server->client stream yet.
    StatusCode::METHOD_NOT_ALLOWED
}

async fn mcp_post(
    State(state): State<HttpState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    if !validate_origin(&headers) {
        return StatusCode::FORBIDDEN.into_response();
    }

    let body_str = match std::str::from_utf8(&body) {
        Ok(s) => s,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    let req: RpcRequest = match serde_json::from_str(body_str) {
        Ok(r) => r,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    // Notifications/responses from client -> accept with 202 if we can parse them.
    let Some(id) = req.id.clone() else {
        return StatusCode::ACCEPTED.into_response();
    };

    // If this request likely needs streaming (progress), respond as SSE.
    let wants_stream = req.method == "tools/call"
        && req
            .params
            .get("_meta")
            .and_then(|m| m.get("progressToken"))
            .is_some();

    if wants_stream {
        let (tx, rx) = mpsc::channel::<Value>(1024);
        let notifier: Arc<dyn Notifier> = Arc::new(ChannelNotifier { tx: tx.clone() });
        let registry = state.registry.clone();

        tokio::spawn(async move {
            let resp = handle_request(registry, id.clone(), req, notifier.clone()).await;
            let _ = tx.send(serde_json::to_value(resp).unwrap_or_else(|_| json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32000, "message": "failed to serialize response" }
            }))).await;
        });

        let stream = ReceiverStream::new(rx).map(|msg| {
            let text = serde_json::to_string(&msg).unwrap_or_else(|_| "{\"jsonrpc\":\"2.0\",\"method\":\"notifications/progress\",\"params\":{\"message\":\"serialization failed\",\"progress\":0,\"progressToken\":\"unknown\"}}".to_string());
            Ok::<Event, Infallible>(Event::default().data(text))
        });

        let mut resp = Sse::new(stream).into_response();
        // Tell clients we support the negotiated protocol version if present.
        if let Some(pv) = headers.get("mcp-protocol-version").cloned() {
            resp.headers_mut()
                .insert("mcp-protocol-version", pv);
        } else {
            resp.headers_mut().insert(
                "mcp-protocol-version",
                HeaderValue::from_static("2025-06-18"),
            );
        }
        return resp;
    }

    // Non-streaming: standard JSON response.
    let (tx, _rx) = mpsc::channel::<Value>(1);
    let notifier: Arc<dyn Notifier> = Arc::new(ChannelNotifier { tx });
    let resp = handle_request(state.registry.clone(), id, req, notifier).await;
    Json(resp).into_response()
}

/// Dispatches methods like `tools/list` and `tools/call`.
async fn handle_request(
    registry: Arc<ToolRegistry>,
    id: Value,
    req: RpcRequest,
    notifier: Arc<dyn Notifier>,
) -> RpcResponse {
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

            let ctx = ExecutionContext::new(id.clone(), params.meta, notifier);
            match registry.call_with_ctx(&params.name, params.input, ctx).await {
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
