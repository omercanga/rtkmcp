use serde_json::{json, Value};

use super::types::{CallToolResult, InitializeResult, Capabilities, ToolsCapability,
                   ServerInfo, ListToolsResult, Request, Response};
use crate::tools;

pub fn dispatch(req: Request) -> Response {
    match req.method.as_str() {
        "initialize" => handle_initialize(req),
        "notifications/initialized" => Response::notification(),
        "ping" => Response::ok(req.id, json!({})),
        "tools/list" => handle_list_tools(req),
        "tools/call" => handle_call_tool(req),
        _ => Response::err(req.id, -32601, format!("Method not found: {}", req.method)),
    }
}

pub fn parse_error(msg: &str) -> Response {
    Response::err(Value::Null, -32700, format!("Parse error: {}", msg))
}

fn handle_initialize(req: Request) -> Response {
    let result = InitializeResult {
        protocol_version: "2024-11-05",
        capabilities: Capabilities { tools: ToolsCapability { list_changed: false } },
        server_info: ServerInfo { name: "rtkmcp", version: env!("CARGO_PKG_VERSION") },
    };
    Response::ok(req.id, serde_json::to_value(result).unwrap())
}

fn handle_list_tools(req: Request) -> Response {
    let result = ListToolsResult { tools: tools::registry::all_tools() };
    Response::ok(req.id, serde_json::to_value(result).unwrap())
}

fn handle_call_tool(req: Request) -> Response {
    let name = match req.params.get("name").and_then(|v| v.as_str()) {
        Some(n) => n.to_string(),
        None => return Response::err(req.id, -32602, "Missing tool name"),
    };

    let args = req.params.get("arguments").cloned().unwrap_or(json!({}));

    let result: CallToolResult = match name.as_str() {
        "shell_run"   => tools::shell_run::call(&args),
        "errors_only" => tools::shell_run::errors_only(&args),
        "read_file"   => tools::read_file::call(&args),
        "git_status"  => tools::git::status(&args),
        "git_log"     => tools::git::log(&args),
        "git_diff"    => tools::git::diff(&args),
        "list_dir"    => tools::dir::list(&args),
        "search_code" => tools::search::call(&args),
        unknown => {
            return Response::err(req.id, -32602, format!("Unknown tool: {}", unknown));
        }
    };

    Response::ok(req.id, serde_json::to_value(result).unwrap())
}
