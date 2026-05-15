use crate::filters::truncate;
use crate::protocol::types::CallToolResult;
use serde_json::Value;
use std::collections::BTreeMap;
use std::process::Command;

pub fn call(args: &Value) -> CallToolResult {
    let pattern = match args.get("pattern").and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => return CallToolResult::error("Missing required argument: pattern"),
    };

    let path        = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");
    let file_type   = args.get("file_type").and_then(|v| v.as_str());
    let max_results = args.get("max_results").and_then(|v| v.as_u64()).unwrap_or(50) as usize;

    let raw = match run_search(&pattern, path, file_type) {
        Ok(r)  => r,
        Err(e) => return CallToolResult::error(format!("Search failed: {}", e)),
    };

    if raw.trim().is_empty() {
        return CallToolResult::text(format!("0 matches for '{}'", pattern));
    }

    let output = group_results(&raw, &pattern, max_results);
    CallToolResult::text(output)
}

fn run_search(pattern: &str, path: &str, file_type: Option<&str>) -> anyhow::Result<String> {
    // Try rg first, fall back to grep
    let mut cmd = Command::new("rg");
    cmd.args(["-n", "--no-heading", pattern, path]);
    if let Some(ft) = file_type {
        cmd.arg("--type").arg(ft);
    }

    let output = cmd.output();

    match output {
        Ok(o) if o.status.code() != Some(127) => {
            Ok(String::from_utf8_lossy(&o.stdout).into_owned())
        }
        _ => {
            // Fall back to grep
            let mut grep = Command::new("grep");
            grep.args(["-rn", pattern, path]);
            if let Some(ext) = file_type {
                grep.arg("--include").arg(format!("*.{}", ext));
            }
            let o = grep.output()?;
            Ok(String::from_utf8_lossy(&o.stdout).into_owned())
        }
    }
}

fn group_results(raw: &str, pattern: &str, max_results: usize) -> String {
    // Parse file:line:content format
    let mut by_file: BTreeMap<String, Vec<(usize, String)>> = BTreeMap::new();

    for line in raw.lines() {
        let parts: Vec<&str> = line.splitn(3, ':').collect();
        if parts.len() < 3 { continue; }

        let file    = parts[0].to_string();
        let line_no = parts[1].parse::<usize>().unwrap_or(0);
        let content = truncate::line(parts[2].trim(), 120);

        by_file.entry(file).or_default().push((line_no, content));
    }

    let total_matches: usize = by_file.values().map(|v| v.len()).sum();
    let file_count = by_file.len();

    let mut out = format!("{} matches in {} file{}:\n",
        total_matches, file_count, if file_count == 1 { "" } else { "s" });

    let per_file_limit = (max_results / file_count.max(1)).max(3);
    let mut shown = 0usize;

    for (file, matches) in &by_file {
        if shown >= max_results { break; }
        out.push_str(&format!("\n{}:\n", file));

        for (ln, content) in matches.iter().take(per_file_limit) {
            out.push_str(&format!("  {}:  {}\n", ln, content));
            shown += 1;
            if shown >= max_results { break; }
        }

        let hidden = matches.len().saturating_sub(per_file_limit);
        if hidden > 0 {
            out.push_str(&format!("  [+{} more in this file]\n", hidden));
        }
    }

    if total_matches > max_results {
        out.push_str(&format!(
            "\n[{} total matches — use a more specific pattern to narrow results]",
            total_matches
        ));
    }

    // Hint: pattern context
    out.push_str(&format!("\n[pattern: '{}']", pattern));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn missing_pattern_returns_error() {
        let res = call(&json!({ "path": "." }));
        assert!(res.is_error);
    }
}
