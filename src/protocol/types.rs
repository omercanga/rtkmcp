use serde::{Deserialize, Serialize};
use serde_json::Value;

// ── JSON-RPC 2.0 ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct Request {
    #[allow(dead_code)]
    pub jsonrpc: String,
    #[serde(default)]
    pub id: Value,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Serialize)]
pub struct Response {
    pub jsonrpc: String,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
}

#[derive(Debug, Serialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
}

impl Response {
    pub fn ok(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn err(id: Value, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: None,
            error: Some(RpcError {
                code,
                message: message.into(),
            }),
        }
    }

    pub fn notification() -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id: Value::Null,
            result: None,
            error: None,
        }
    }
}

// ── MCP Protocol ─────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct ServerInfo {
    pub name: &'static str,
    pub version: &'static str,
}

#[derive(Serialize)]
pub struct Capabilities {
    pub tools: ToolsCapability,
}

#[derive(Serialize)]
pub struct ToolsCapability {
    #[serde(rename = "listChanged")]
    pub list_changed: bool,
}

#[derive(Serialize)]
pub struct InitializeResult {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: &'static str,
    pub capabilities: Capabilities,
    #[serde(rename = "serverInfo")]
    pub server_info: ServerInfo,
}

#[derive(Serialize, Clone)]
pub struct Tool {
    pub name: &'static str,
    pub description: &'static str,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

#[derive(Serialize)]
pub struct ListToolsResult {
    pub tools: Vec<Tool>,
}

#[derive(Serialize)]
pub struct CallToolResult {
    pub content: Vec<Content>,
    #[serde(rename = "isError")]
    pub is_error: bool,
}

#[derive(Serialize)]
pub struct Content {
    pub r#type: &'static str,
    pub text: String,
}

impl CallToolResult {
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            content: vec![Content {
                r#type: "text",
                text: text.into(),
            }],
            is_error: false,
        }
    }

    pub fn error(text: impl Into<String>) -> Self {
        Self {
            content: vec![Content {
                r#type: "text",
                text: text.into(),
            }],
            is_error: true,
        }
    }
}
