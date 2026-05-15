use crate::protocol::types::CallToolResult;
use serde_json::Value;
use std::fs;
use std::path::Path;

const SKIP_DIRS: &[&str] = &[
    "node_modules", ".git", "target", ".next", "dist", "build",
    "__pycache__", ".pytest_cache", ".mypy_cache", "vendor", ".cargo",
];

pub fn list(args: &Value) -> CallToolResult {
    let path_str = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");
    let depth    = args.get("depth").and_then(|v| v.as_u64()).unwrap_or(2) as usize;
    let depth    = depth.min(4); // hard cap — deep trees are expensive

    let path = Path::new(path_str);
    if !path.exists() {
        return CallToolResult::error(format!("Path not found: {}", path_str));
    }
    if !path.is_dir() {
        return CallToolResult::error(format!("Not a directory: {}", path_str));
    }

    let mut out = format!("{}/\n", path.display());
    let mut file_count = 0usize;
    let mut dir_count  = 0usize;

    walk(path, 1, depth, &mut out, &mut file_count, &mut dir_count);

    out.push_str(&format!(
        "\n{} files, {} directories",
        file_count, dir_count
    ));

    CallToolResult::text(out)
}

fn walk(
    dir: &Path, current_depth: usize, max_depth: usize,
    out: &mut String, file_count: &mut usize, dir_count: &mut usize,
) {
    let indent = "  ".repeat(current_depth);

    let mut entries: Vec<fs::DirEntry> = match fs::read_dir(dir) {
        Ok(rd) => rd.filter_map(|e| e.ok()).collect(),
        Err(_) => return,
    };

    // Dirs first, then files, both alphabetical
    entries.sort_by_key(|e| {
        let is_file = e.path().is_file();
        (is_file as u8, e.file_name())
    });

    let mut file_lines: Vec<String> = Vec::new();

    for entry in &entries {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        let path = entry.path();

        if path.is_dir() {
            if SKIP_DIRS.contains(&name_str.as_ref()) {
                out.push_str(&format!("{}{}/ [skipped]\n", indent, name_str));
                continue;
            }
            *dir_count += 1;
            out.push_str(&format!("{}{}/\n", indent, name_str));
            if current_depth < max_depth {
                walk(&path, current_depth + 1, max_depth, out, file_count, dir_count);
            } else {
                // Count children without showing them
                let n = fs::read_dir(&path).map(|r| r.count()).unwrap_or(0);
                if n > 0 {
                    out.push_str(&format!("{}  [{}+ items]\n", indent, n));
                }
            }
        } else {
            *file_count += 1;
            let size = human_size(entry.metadata().map(|m| m.len()).unwrap_or(0));
            file_lines.push(format!("{}{}  {}", indent, name_str, size));
        }
    }

    // Batch file lines — collapse if >8 files at this level
    if file_lines.len() <= 8 {
        for line in &file_lines {
            out.push_str(line);
            out.push('\n');
        }
    } else {
        for line in file_lines.iter().take(6) {
            out.push_str(line);
            out.push('\n');
        }
        out.push_str(&format!("{}[+{} more files]\n", indent, file_lines.len() - 6));
    }
}

fn human_size(bytes: u64) -> String {
    if bytes >= 1_048_576 {
        format!("{:.1}MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1_024 {
        format!("{:.1}KB", bytes as f64 / 1_024.0)
    } else {
        format!("{}B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn nonexistent_path_error() {
        let res = list(&json!({ "path": "/no/such/dir" }));
        assert!(res.is_error);
    }

    #[test]
    fn human_size_formatting() {
        assert_eq!(human_size(0), "0B");
        assert_eq!(human_size(1024), "1.0KB");
        assert_eq!(human_size(1_048_576), "1.0MB");
    }
}
