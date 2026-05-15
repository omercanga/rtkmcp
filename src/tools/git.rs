use crate::filters::git as gf;
use crate::protocol::types::CallToolResult;
use serde_json::Value;
use std::process::Command;

pub fn status(args: &Value) -> CallToolResult {
    let cwd = args.get("cwd").and_then(|v| v.as_str());

    match git_run(&["status", "--porcelain", "-b", "-uall"], cwd) {
        Ok(out) => {
            let filtered = gf::compact_status(&out);
            CallToolResult::text(filtered)
        }
        Err(e) => CallToolResult::error(format!("git status failed: {}", e)),
    }
}

pub fn log(args: &Value) -> CallToolResult {
    let cwd = args.get("cwd").and_then(|v| v.as_str());
    let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(20) as usize;
    let extra: Vec<String> = args
        .get("args")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();

    let mut git_args = vec![
        "log".to_string(),
        "--pretty=format:%h %s (%ar) <%an>%n%b%n---END---".to_string(),
        format!("-{}", limit),
        "--no-merges".to_string(),
    ];
    git_args.extend(extra);

    let arg_refs: Vec<&str> = git_args.iter().map(String::as_str).collect();

    match git_run(&arg_refs, cwd) {
        Ok(out) => {
            let filtered = gf::compact_log(&out, limit);
            CallToolResult::text(filtered)
        }
        Err(e) => CallToolResult::error(format!("git log failed: {}", e)),
    }
}

pub fn diff(args: &Value) -> CallToolResult {
    let cwd = args.get("cwd").and_then(|v| v.as_str());
    let max_lines = args
        .get("max_lines")
        .and_then(|v| v.as_u64())
        .unwrap_or(300) as usize;
    let extra: Vec<String> = args
        .get("args")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();

    // First: get stat summary
    let mut stat_args = vec!["diff", "--stat"];
    let extra_refs: Vec<&str> = extra.iter().map(String::as_str).collect();
    stat_args.extend(extra_refs.iter().copied());

    let stat_out = git_run(&stat_args, cwd).unwrap_or_default();

    // Then: get actual diff and compact it
    let mut diff_args = vec!["diff"];
    diff_args.extend(extra_refs.iter().copied());

    match git_run(&diff_args, cwd) {
        Ok(raw_diff) => {
            let compacted = gf::compact_diff(&raw_diff, max_lines);
            let output = if stat_out.trim().is_empty() {
                compacted
            } else {
                format!("{}\n\n---\n{}", stat_out.trim(), compacted)
            };
            CallToolResult::text(output)
        }
        Err(e) => CallToolResult::error(format!("git diff failed: {}", e)),
    }
}

fn git_run(args: &[&str], cwd: Option<&str>) -> anyhow::Result<String> {
    let mut cmd = Command::new("git");
    cmd.args(args);
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }
    let output = cmd.output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("{}", stderr.trim());
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}
