//! Build/test output filter — errors-only extraction.
//! Ported from RTK cmds/rust/cargo_cmd.rs BlockStreamFilter logic.

use lazy_static::lazy_static;
use regex::Regex;

#[derive(Debug, Clone, Copy)]
pub enum Tool {
    Cargo,
    Npm,
    Go,
    Dotnet,
    Generic,
}

impl Tool {
    pub fn detect(command: &str) -> Self {
        let cmd = command.split_whitespace().next().unwrap_or("");
        match cmd {
            "cargo"              => Self::Cargo,
            "npm" | "pnpm" | "yarn" | "bun" => Self::Npm,
            "go"                 => Self::Go,
            "dotnet"             => Self::Dotnet,
            _                    => Self::Generic,
        }
    }
}

pub struct FilterResult {
    pub text: String,
    #[allow(dead_code)]
    pub error_count: usize,
    #[allow(dead_code)]
    pub passed_count: usize,
}

lazy_static! {
    // Lines to drop (carry zero LLM value)
    static ref SKIP: Regex = Regex::new(
        r"(?x)^(
            Compiling\s | Downloading\s | Downloaded\s
            | Blocking\s+waiting | Locking\s+\d | Fresh\s
            | Finished\s | Building\s
            | test\s+\S+\s+\.\.\.\s+ok   # passing test
            | ✓ | ✔ | PASS\s             # passing test icons
            | running\s+\d+\s+test
            | \s*$                        # blank
        )"
    ).unwrap();

    // Error block starters
    static ref ERROR_START: Regex = Regex::new(
        r"(?x)^(
            error(\[E\d+\])? | FAILED | FAIL\s
            | panic! | npm\s+ERR! | Error:
            | --- FAIL | FAILED\s+\(
        )"
    ).unwrap();

    // Error block continuation (indented context / arrows)
    static ref ERROR_CONT: Regex = Regex::new(
        r"^(\s+|\s*-->\s|\s*\|\s|\s*=\s|\s*\^)"
    ).unwrap();

    // Test summary lines (always keep)
    static ref SUMMARY: Regex = Regex::new(
        r"(?x)(
            test\s+result: | FAILED\s*\(failures= | \d+\s+passed
            | \d+\s+failed | \d+\s+ignored
            | ok\.\s+\d+ | failures: | \d+\s+tests? run
        )"
    ).unwrap();
}

/// Filter `raw` output, keeping only errors, failures, and their context.
/// Returns a `FilterResult` with the condensed text and counts.
pub fn errors_only(raw: &str, tool: Tool) -> FilterResult {
    let clean = super::ansi::strip(raw);

    let mut blocks: Vec<Vec<String>> = Vec::new();
    let mut current_block: Vec<String> = Vec::new();
    let mut in_block = false;
    let mut pass_count = 0usize;
    let mut summaries: Vec<String> = Vec::new();

    for line in clean.lines() {
        let t = line.trim();

        // Always collect summary lines separately
        if SUMMARY.is_match(t) {
            if let Some(counts) = extract_pass_count(t) {
                pass_count += counts;
            }
            if !in_block {
                summaries.push(line.to_string());
                continue;
            }
        }

        if SKIP.is_match(t) && !in_block {
            continue;
        }

        if ERROR_START.is_match(t) {
            // Flush previous block
            if !current_block.is_empty() {
                blocks.push(std::mem::take(&mut current_block));
            }
            current_block.push(line.to_string());
            in_block = true;
            continue;
        }

        if in_block {
            if ERROR_CONT.is_match(line) || t.is_empty() {
                current_block.push(line.to_string());
            } else {
                blocks.push(std::mem::take(&mut current_block));
                in_block = false;
                // Don't skip this line — check if it starts another block
                if ERROR_START.is_match(t) {
                    current_block.push(line.to_string());
                    in_block = true;
                }
            }
        }
    }
    if !current_block.is_empty() {
        blocks.push(current_block);
    }

    let error_count = blocks.len();
    let max_blocks = 15usize;

    let mut text = String::new();

    if blocks.is_empty() {
        // No errors — show pass summary
        let summary = if pass_count > 0 {
            format!("OK — {} test{} passed", pass_count, if pass_count == 1 { "" } else { "s" })
        } else {
            format!("OK ({})", tool_name(tool))
        };
        text.push_str(&summary);
    } else {
        text.push_str(&format!("{} error{}:\n", error_count, if error_count == 1 { "" } else { "s" }));

        for (i, block) in blocks.iter().take(max_blocks).enumerate() {
            if i > 0 { text.push('\n'); }
            for line in block {
                text.push_str(line);
                text.push('\n');
            }
        }

        if error_count > max_blocks {
            text.push_str(&format!("\n[+{} more errors]\n", error_count - max_blocks));
        }
    }

    // Append summaries
    for s in &summaries {
        if !text.contains(s.trim()) {
            text.push('\n');
            text.push_str(s);
        }
    }

    FilterResult { text: text.trim_end().to_string(), error_count, passed_count: pass_count }
}

fn extract_pass_count(line: &str) -> Option<usize> {
    // "47 passed" or "test result: ok. 47 passed"
    lazy_static! {
        static ref PASS_RE: Regex = Regex::new(r"(\d+)\s+passed").unwrap();
    }
    PASS_RE.captures(line)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse().ok())
}

fn tool_name(tool: Tool) -> &'static str {
    match tool {
        Tool::Cargo   => "cargo",
        Tool::Npm     => "npm",
        Tool::Go      => "go",
        Tool::Dotnet  => "dotnet",
        Tool::Generic => "command",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_compile_noise() {
        let raw = "Compiling foo v1.0\nerror[E0308]: type mismatch\n  --> src/main.rs:10\n";
        let res = errors_only(raw, Tool::Cargo);
        assert!(!res.text.contains("Compiling"));
        assert!(res.text.contains("error[E0308]"));
        assert_eq!(res.error_count, 1);
    }

    #[test]
    fn all_pass_returns_ok() {
        let raw = "running 5 tests\ntest a ... ok\ntest b ... ok\ntest result: ok. 5 passed; 0 failed\n";
        let res = errors_only(raw, Tool::Cargo);
        assert_eq!(res.error_count, 0);
        assert!(res.text.contains("OK"));
    }
}
