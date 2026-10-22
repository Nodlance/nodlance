//! AI-Native Model Context Protocol (MCP) server for Nodlance Database Control Plane.
//!
//! Provides stdio-based JSON-RPC 2.0 communication enabling Claude Code, Antigravity,
//! Cursor, and LLM DBA agents to inspect topology, execute Rhai advisor scans,
//! analyze config diffs, and orchestrate safe automated failovers.

pub mod tools;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::io::{self, BufRead, Write};

pub use tools::{McpTool, execute_tool, get_available_tools};

/// JSON-RPC 2.0 Request envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

/// JSON-RPC 2.0 Response envelope.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcResponse {
    pub fn success(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Value, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data: None,
            }),
        }
    }
}

/// Dispatches an incoming JSON-RPC 2.0 request according to the MCP protocol.
pub async fn handle_request(req: JsonRpcRequest) -> Option<JsonRpcResponse> {
    let id = req.id.clone();

    match req.method.as_str() {
        // MCP Handshake
        "initialize" => {
            let id = id.unwrap_or(json!(1));
            Some(JsonRpcResponse::success(
                id,
                json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {
                            "listChanged": false
                        }
                    },
                    "serverInfo": {
                        "name": "nodlance-mcp",
                        "version": env!("CARGO_PKG_VERSION")
                    }
                }),
            ))
        }

        // Notifications without ID
        "notifications/initialized" => None,

        // Liveness Ping
        "ping" => {
            let id = id.unwrap_or(json!(1));
            Some(JsonRpcResponse::success(id, json!({})))
        }

        // List Available Tools
        "tools/list" => {
            let id = id.unwrap_or(json!(1));
            let tools = get_available_tools();
            Some(JsonRpcResponse::success(id, json!({ "tools": tools })))
        }

        // Call Tool
        "tools/call" => {
            let id = id.unwrap_or(json!(1));
            let tool_name = req.params.get("name").and_then(|v| v.as_str());
            let arguments = req.params.get("arguments").cloned().unwrap_or(json!({}));

            match tool_name {
                Some(name) => match execute_tool(name, arguments).await {
                    Ok(text_content) => Some(JsonRpcResponse::success(
                        id,
                        json!({
                            "content": [
                                {
                                    "type": "text",
                                    "text": text_content
                                }
                            ],
                            "isError": false
                        }),
                    )),
                    Err(err_msg) => Some(JsonRpcResponse::success(
                        id,
                        json!({
                            "content": [
                                {
                                    "type": "text",
                                    "text": format!("Error executing tool '{name}': {err_msg}")
                                }
                            ],
                            "isError": true
                        }),
                    )),
                },
                None => Some(JsonRpcResponse::error(
                    id,
                    -32602,
                    "Invalid params: missing tool 'name'",
                )),
            }
        }

        unknown => {
            let id = id.unwrap_or(json!(1));
            Some(JsonRpcResponse::error(
                id,
                -32601,
                format!("Method not found: '{unknown}'"),
            ))
        }
    }
}

/// Runs the MCP stdio loop over standard input and standard output.
pub async fn run_stdio_server() -> io::Result<()> {
    tokio::task::spawn_blocking(|| {
        let stdin = io::stdin();
        let mut handle = stdin.lock();
        let mut stdout = io::stdout();
        let mut line = String::new();

        while handle.read_line(&mut line)? > 0 {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                if let Ok(req) = serde_json::from_str::<JsonRpcRequest>(trimmed) {
                    let runtime = tokio::runtime::Handle::current();
                    if let Some(resp) = runtime.block_on(handle_request(req)) {
                        if let Ok(payload) = serde_json::to_string(&resp) {
                            let _ = writeln!(stdout, "{payload}");
                            let _ = stdout.flush();
                        }
                    }
                }
            }
            line.clear();
        }
        Ok::<(), io::Error>(())
    })
    .await?
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mcp_initialize() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(10)),
            method: "initialize".to_string(),
            params: json!({ "protocolVersion": "2024-11-05" }),
        };

        let resp = handle_request(req).await.expect("must return response");
        assert_eq!(resp.id, json!(10));
        let res = resp.result.expect("must have result");
        assert_eq!(res["serverInfo"]["name"], "nodlance-mcp");
    }

    #[tokio::test]
    async fn test_mcp_tools_list() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(20)),
            method: "tools/list".to_string(),
            params: json!({}),
        };

        let resp = handle_request(req).await.expect("must return response");
        let tools_list = resp.result.unwrap()["tools"].as_array().unwrap().clone();
        assert_eq!(tools_list.len(), 7);

        let names: Vec<&str> = tools_list
            .iter()
            .map(|t| t["name"].as_str().unwrap())
            .collect();

        assert!(names.contains(&"nodlance_get_fleet_summary"));
        assert!(names.contains(&"nodlance_inspect_cluster"));
        assert!(names.contains(&"nodlance_run_advisor_scan"));
        assert!(names.contains(&"nodlance_get_slow_queries"));
        assert!(names.contains(&"nodlance_calculate_config_diff"));
        assert!(names.contains(&"nodlance_plan_failover"));
        assert!(names.contains(&"nodlance_trigger_backup"));
    }

    #[tokio::test]
    async fn test_mcp_tool_call_fleet_summary() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(30)),
            method: "tools/call".to_string(),
            params: json!({
                "name": "nodlance_get_fleet_summary",
                "arguments": {}
            }),
        };

        let resp = handle_request(req).await.expect("must return response");
        let content = &resp.result.unwrap()["content"][0]["text"];
        assert!(content.as_str().unwrap().contains("c1a7001-pg16-prod"));
    }

    #[tokio::test]
    async fn test_mcp_tool_call_advisor_scan() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(40)),
            method: "tools/call".to_string(),
            params: json!({
                "name": "nodlance_run_advisor_scan",
                "arguments": { "cluster_id": "c1a7001-pg16-prod" }
            }),
        };

        let resp = handle_request(req).await.expect("must return response");
        let content = &resp.result.unwrap()["content"][0]["text"];
        assert!(content.as_str().unwrap().contains("Rhai 1.27"));
    }
}
