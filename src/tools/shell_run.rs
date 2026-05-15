use crate::filters::{build, noise};
use crate::protocol::types::CallToolResult;
use serde_json::Value;
use std::process::Command;

/// Run a command and return noise-filtered output.
pub fn call(args: &Value) -> CallToolResult {
    let command = match args.get("command").and_then(|v| v.as_str()) {
        Some(c) => c.to_string(),
        None => return CallToolResult::error("Missing required argument: command"),
    };
    let cwd = args.get("cwd").and_then(|v| v.as_str());
    let filter = args
        .get("filter")
        .and_then(|v| v.as_str())
        .unwrap_or("auto");

    match run_command(&command, cwd) {
        Ok((stdout, stderr, code)) => {
            let raw = combine_output(&stdout, &stderr);
            let output = if filter == "none" {
                raw.clone()
            } else {
                noise::reduce(&raw)
            };
            let output = if output.trim().is_empty() && code == 0 {
                "OK (exit 0)\n[command produced no output]".to_string()
            } else {
                output
            };
            if code != 0 && output.trim().is_empty() {
                CallToolResult::error(format!("Command failed (exit {})\n{}", code, raw))
            } else {
                CallToolResult::text(output)
            }
        }
        Err(e) => CallToolResult::error(format!("Failed to run command: {}", e)),
    }
}

/// Run a build/test command and return ONLY errors/failures.
pub fn errors_only(args: &Value) -> CallToolResult {
    let command = match args.get("command").and_then(|v| v.as_str()) {
        Some(c) => c.to_string(),
        None => return CallToolResult::error("Missing required argument: command"),
    };
    let cwd = args.get("cwd").and_then(|v| v.as_str());
    let tool = build::Tool::detect(&command);

    match run_command(&command, cwd) {
        Ok((stdout, stderr, _code)) => {
            let raw = combine_output(&stdout, &stderr);
            let result = build::errors_only(&raw, tool);
            CallToolResult::text(result.text)
        }
        Err(e) => CallToolResult::error(format!("Failed to run command: {}", e)),
    }
}

fn run_command(command: &str, cwd: Option<&str>) -> anyhow::Result<(String, String, i32)> {
    let parts = shell_split(command);
    if parts.is_empty() {
        anyhow::bail!("Empty command");
    }

    let mut cmd = Command::new(&parts[0]);
    cmd.args(&parts[1..]);

    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }

    let output = cmd.output()?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let code = output.status.code().unwrap_or(1);

    Ok((stdout, stderr, code))
}

fn combine_output(stdout: &str, stderr: &str) -> String {
    match (stdout.trim().is_empty(), stderr.trim().is_empty()) {
        (true, true) => String::new(),
        (true, false) => stderr.to_string(),
        (false, true) => stdout.to_string(),
        (false, false) => format!("{}\n{}", stdout.trim_end(), stderr.trim_end()),
    }
}

/// Minimal shell tokenizer — handles quoted strings, no expansion.
fn shell_split(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_single = false;
    let mut in_double = false;

    for ch in input.chars() {
        match ch {
            '\'' if !in_double => in_single = !in_single,
            '"' if !in_single => in_double = !in_double,
            ' ' | '\t' if !in_single && !in_double => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(ch),
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_split_basic() {
        assert_eq!(
            shell_split("cargo build --release"),
            vec!["cargo", "build", "--release"]
        );
    }

    #[test]
    fn shell_split_quoted() {
        assert_eq!(
            shell_split(r#"git log --format="%H %s""#),
            vec!["git", "log", "--format=%H %s"]
        );
    }
}
