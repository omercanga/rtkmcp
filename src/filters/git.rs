//! Git output filters — ported and adapted from RTK cmds/git/git.rs

use super::truncate;

/// Compact git status from porcelain -b output.
/// Input:  `## main...origin/main\nM  src/foo.rs\n?? bar.rs`
/// Output: `* main...origin/main\nM  src/foo.rs\n?? bar.rs`
pub fn compact_status(porcelain: &str) -> String {
    let lines: Vec<&str> = porcelain.lines().filter(|l| !l.trim().is_empty()).collect();

    if lines.is_empty() {
        return "clean — nothing to commit".to_string();
    }

    let mut out: Vec<String> = Vec::new();

    if let Some(first) = lines.first() {
        if first.starts_with("##") {
            out.push(format!("* {}", first.trim_start_matches("## ")));
        } else {
            out.push((*first).to_string());
        }
    }

    // If only the branch line, nothing changed
    if lines.len() == 1 && lines[0].starts_with("##") {
        out.push("clean — nothing to commit".to_string());
        return out.join("\n");
    }

    for line in lines.iter().skip(1) {
        out.push((*line).to_string());
    }

    out.join("\n")
}

/// Compact git log from RTK-injected `---END---`-separated blocks.
/// Falls back to simple line truncation when no markers present.
pub fn compact_log(raw: &str, limit: usize) -> String {
    if raw.contains("---END---") {
        compact_log_blocks(raw, limit)
    } else {
        compact_log_lines(raw, limit)
    }
}

fn compact_log_blocks(raw: &str, limit: usize) -> String {
    let mut out: Vec<String> = Vec::new();

    for block in raw.split("---END---").take(limit) {
        let block = block.trim();
        if block.is_empty() { continue; }

        let mut lines = block.lines();
        let header = match lines.next() {
            Some(h) => truncate::line(h.trim(), 100),
            None    => continue,
        };

        let body: Vec<&str> = lines
            .map(|l| l.trim())
            .filter(|l| {
                !l.is_empty()
                    && !l.starts_with("Signed-off-by:")
                    && !l.starts_with("Co-authored-by:")
            })
            .collect();

        if body.is_empty() {
            out.push(header);
        } else {
            let shown = body.len().min(3);
            let omitted = body.len().saturating_sub(shown);
            let mut entry = header;
            for b in &body[..shown] {
                entry.push_str(&format!("\n  {}", truncate::line(b, 100)));
            }
            if omitted > 0 {
                entry.push_str(&format!("\n  [+{} lines omitted]", omitted));
            }
            out.push(entry);
        }
    }

    out.join("\n").trim().to_string()
}

fn compact_log_lines(raw: &str, limit: usize) -> String {
    raw.lines()
        .take(limit)
        .map(|l| truncate::line(l, 100))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Compact git diff: keeps file headers, hunk headers (@@), and +/- lines.
/// Context lines are included up to `max_hunk_lines` per hunk.
pub fn compact_diff(diff: &str, max_lines: usize) -> String {
    let mut result: Vec<String> = Vec::new();
    let mut current_file = String::new();
    let mut added   = 0i32;
    let mut removed = 0i32;
    let mut in_hunk = false;
    let mut hunk_shown   = 0usize;
    let mut hunk_skipped = 0usize;
    let max_hunk_lines   = 100usize;
    let mut was_truncated = false;

    for line in diff.lines() {
        if line.starts_with("diff --git") {
            flush_hunk_skip(&mut result, &mut hunk_skipped, &mut was_truncated);
            flush_file_stats(&mut result, &current_file, added, removed);
            current_file = line.split(" b/").nth(1).unwrap_or("unknown").to_string();
            result.push(format!("\n{}", current_file));
            added = 0; removed = 0; in_hunk = false; hunk_shown = 0;

        } else if line.starts_with("@@") {
            flush_hunk_skip(&mut result, &mut hunk_skipped, &mut was_truncated);
            in_hunk    = true;
            hunk_shown = 0;
            result.push(format!("  {}", line));

        } else if in_hunk {
            if line.starts_with('+') && !line.starts_with("+++") {
                added += 1;
                push_hunk_line(&mut result, line, &mut hunk_shown, &mut hunk_skipped, max_hunk_lines);
            } else if line.starts_with('-') && !line.starts_with("---") {
                removed += 1;
                push_hunk_line(&mut result, line, &mut hunk_shown, &mut hunk_skipped, max_hunk_lines);
            } else if !line.starts_with('\\') && hunk_shown > 0 && hunk_shown < max_hunk_lines {
                result.push(format!("  {}", line));
                hunk_shown += 1;
            }
        }

        if result.len() >= max_lines {
            result.push("\n... (more changes truncated)".to_string());
            was_truncated = true;
            break;
        }
    }

    flush_hunk_skip(&mut result, &mut hunk_skipped, &mut was_truncated);
    flush_file_stats(&mut result, &current_file, added, removed);

    if was_truncated {
        result.push("[full diff: git diff --no-compact]".to_string());
    }

    result.join("\n")
}

fn push_hunk_line(
    result: &mut Vec<String>, line: &str,
    shown: &mut usize, skipped: &mut usize, max: usize,
) {
    if *shown < max {
        result.push(format!("  {}", line));
        *shown += 1;
    } else {
        *skipped += 1;
    }
}

fn flush_hunk_skip(result: &mut Vec<String>, skipped: &mut usize, was_truncated: &mut bool) {
    if *skipped > 0 {
        result.push(format!("  ... ({} lines truncated)", skipped));
        *was_truncated = true;
        *skipped = 0;
    }
}

fn flush_file_stats(result: &mut Vec<String>, file: &str, added: i32, removed: i32) {
    if !file.is_empty() && (added > 0 || removed > 0) {
        result.push(format!("  +{} -{}", added, removed));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_clean_tree() {
        assert_eq!(compact_status(""), "clean — nothing to commit");
    }

    #[test]
    fn status_with_changes() {
        let p = "## main...origin/main\nM  src/foo.rs\n?? new.rs\n";
        let out = compact_status(p);
        assert!(out.starts_with("* main"));
        assert!(out.contains("M  src/foo.rs"));
        assert!(out.contains("?? new.rs"));
    }

    #[test]
    fn log_simple_lines() {
        let raw = "abc1234 feat: add thing\ndef5678 fix: bug\n";
        let out = compact_log(raw, 10);
        assert!(out.contains("abc1234"));
        assert!(out.contains("def5678"));
    }

    #[test]
    fn diff_extracts_file_header() {
        let diff = "diff --git a/src/foo.rs b/src/foo.rs\n@@  -1,3 +1,4 @@\n+new line\n";
        let out = compact_diff(diff, 500);
        assert!(out.contains("src/foo.rs"));
        assert!(out.contains("+new line"));
    }
}
