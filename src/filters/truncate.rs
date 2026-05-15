//! Smart truncation — ported from RTK core/filter.rs smart_truncate

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref FUNC_SIG: Regex = Regex::new(
        r"^(pub\s+)?(async\s+)?(fn|def|function|func|class|struct|enum|trait|interface|type)\s+\w+"
    )
    .unwrap();
    static ref IMPORT_PAT: Regex = Regex::new(r"^(use |import |from |require\(|#include)").unwrap();
}

/// Truncate `content` to at most `max_lines`, keeping structurally important
/// lines (signatures, imports, braces) and adding a `[N more lines]` marker.
/// Returns content unchanged when under the limit.
pub fn smart(content: &str, max_lines: usize) -> String {
    let lines: Vec<&str> = content.lines().collect();
    if lines.len() <= max_lines {
        return content.to_string();
    }

    let mut kept: Vec<String> = Vec::with_capacity(max_lines + 1);
    let mut kept_count = 0usize;

    for line in &lines {
        let t = line.trim();
        let important = FUNC_SIG.is_match(t)
            || IMPORT_PAT.is_match(t)
            || t.starts_with("pub ")
            || t.starts_with("export ")
            || t == "}"
            || t == "{";

        if important || kept_count < max_lines / 2 {
            kept.push((*line).to_string());
            kept_count += 1;
        }

        if kept_count >= max_lines.saturating_sub(1) {
            break;
        }
    }

    let remaining = lines.len() - kept_count;
    kept.push(format!("[{} more lines]", remaining));
    kept.join("\n")
}

/// Hard truncate: keep first `max_lines` lines + marker. No intelligence.
#[allow(dead_code)]
pub fn hard(content: &str, max_lines: usize) -> String {
    let lines: Vec<&str> = content.lines().collect();
    if lines.len() <= max_lines {
        return content.to_string();
    }
    let mut out = lines[..max_lines].join("\n");
    let remaining = lines.len() - max_lines;
    out.push_str(&format!("\n[{} more lines]", remaining));
    out
}

/// Truncate a single line to `width` chars, appending "..." if needed.
pub fn line(s: &str, width: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= width {
        s.to_string()
    } else if width < 3 {
        "...".to_string()
    } else {
        format!("{}...", chars[..width - 3].iter().collect::<String>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_truncation_when_under_limit() {
        let input = "a\nb\nc\n";
        assert_eq!(smart(input, 10), input);
    }

    #[test]
    fn marker_accounts_for_all_lines() {
        let total = 200usize;
        let max = 20usize;
        let input: String = (0..total)
            .map(|i| format!("line {}", i))
            .collect::<Vec<_>>()
            .join("\n");
        let out = smart(&input, max);
        let marker = out.lines().find(|l| l.contains("more lines")).unwrap();
        let n: usize = marker
            .trim_start_matches('[')
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap();
        let kept = out.lines().filter(|l| !l.contains("more lines")).count();
        assert_eq!(kept + n, total);
    }

    #[test]
    fn line_truncation() {
        assert_eq!(line("hello world", 8), "hello...");
        assert_eq!(line("hi", 10), "hi");
    }
}
