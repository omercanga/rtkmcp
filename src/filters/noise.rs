//! Generic noise reduction for command output.

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    // Lines that carry zero information for LLMs
    static ref NOISE: Regex = Regex::new(
        r"(?x)^(
            \s*$                                              # blank
            | Compiling\s                                     # cargo compile progress
            | Downloading\s                                   # cargo/npm download
            | Downloaded\s
            | Blocking\s+waiting
            | Locking\s+\d+
            | Fresh\s
            | [\s\d]+packages?\s+in\s+[\d.]+s               # npm timing
            | npm\s+warn\s+(?i:deprecated)                   # npm deprecation
            | \[[\d.]+s\]\s*$                                # bare timing line
        )"
    ).unwrap();

    // Progress bar patterns (spinner chars, \r lines, etc.)
    static ref PROGRESS: Regex = Regex::new(r"[\r]|[⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏]").unwrap();

    // Trailing whitespace
    static ref TRAILING_WS: Regex = Regex::new(r"[ \t]+$").unwrap();
}

/// Remove build noise, progress bars, and blank-only lines from output.
/// Never drops error-like lines.
pub fn reduce(raw: &str) -> String {
    let cleaned = super::ansi::strip(raw);
    let cleaned = PROGRESS.replace_all(&cleaned, "");

    let mut out = String::with_capacity(cleaned.len());
    let mut blank_run = 0usize;

    for line in cleaned.lines() {
        let t = line.trim();

        // Always keep error/warning lines
        if is_important(t) {
            blank_run = 0;
            let line = TRAILING_WS.replace_all(line, "");
            out.push_str(&line);
            out.push('\n');
            continue;
        }

        if NOISE.is_match(t) {
            continue;
        }

        if t.is_empty() {
            blank_run += 1;
            if blank_run <= 1 {
                out.push('\n');
            }
            continue;
        }

        blank_run = 0;
        let line = TRAILING_WS.replace_all(line, "");
        out.push_str(&line);
        out.push('\n');
    }

    out.trim_end().to_string()
}

fn is_important(t: &str) -> bool {
    let tl = t.to_lowercase();
    tl.starts_with("error")
        || tl.starts_with("warning")
        || tl.starts_with("fatal")
        || tl.starts_with("fail")
        || tl.starts_with("panic")
        || t.starts_with("-->")
        || t.starts_with("  |")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drops_compiling_lines() {
        let raw = "Compiling foo v1.0\nCompiling bar v2.0\nerror: oops\n";
        let out = reduce(raw);
        assert!(!out.contains("Compiling"));
        assert!(out.contains("error: oops"));
    }

    #[test]
    fn keeps_errors() {
        let raw = "Compiling x\nerror[E0308]: type mismatch\n  --> src/main.rs:10\n";
        let out = reduce(raw);
        assert!(out.contains("error[E0308]"));
        assert!(out.contains("  --> src/main.rs:10"));
    }

    #[test]
    fn collapses_blanks() {
        let raw = "a\n\n\n\n\nb\n";
        let out = reduce(raw);
        assert!(!out.contains("\n\n\n"));
    }
}
