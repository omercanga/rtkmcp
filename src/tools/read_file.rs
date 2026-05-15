use crate::filters::{lang, truncate};
use crate::protocol::types::CallToolResult;
use serde_json::Value;
use std::fs;
use std::path::Path;

pub fn call(args: &Value) -> CallToolResult {
    let path_str = match args.get("path").and_then(|v| v.as_str()) {
        Some(p) => p,
        None => return CallToolResult::error("Missing required argument: path"),
    };

    let level_str = args.get("level").and_then(|v| v.as_str()).unwrap_or("minimal");
    let max_lines = args.get("max_lines").and_then(|v| v.as_u64()).unwrap_or(500) as usize;
    let level = lang::FilterLevel::from_str(level_str);

    let path = Path::new(path_str);

    let content = match fs::read_to_string(path) {
        Ok(c)  => c,
        Err(e) => return CallToolResult::error(format!("Cannot read '{}': {}", path_str, e)),
    };

    if content.trim().is_empty() {
        return CallToolResult::text(format!("[empty file: {}]", path_str));
    }

    let language = lang::Language::from_path(path);
    let filtered = lang::apply(&content, language, level);

    // Fallback: if filter emptied a non-empty file, return raw
    let filtered = if filtered.trim().is_empty() {
        content.clone()
    } else {
        filtered
    };

    let output = truncate::smart(&filtered, max_lines);

    let line_count  = content.lines().count();
    let out_count   = output.lines().count();
    let savings_pct = if line_count > 0 {
        100.0 - (out_count as f64 / line_count as f64 * 100.0)
    } else {
        0.0
    };

    let header = format!(
        "# {} ({} → {} lines, {:.0}% saved)\n",
        path_str, line_count, out_count, savings_pct
    );

    CallToolResult::text(format!("{}{}", header, output))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn missing_path_returns_error() {
        let res = call(&json!({}));
        assert!(res.is_error);
    }

    #[test]
    fn nonexistent_file_error() {
        let res = call(&json!({ "path": "/nonexistent/file.rs" }));
        assert!(res.is_error);
        assert!(res.content[0].text.contains("Cannot read"));
    }
}
