//! Language detection + comment stripping (ported from RTK core/filter.rs)

use lazy_static::lazy_static;
use regex::Regex;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Go,
    C,
    Cpp,
    Java,
    Ruby,
    Shell,
    Data, // JSON/YAML/TOML/XML — never comment-strip
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterLevel {
    None,
    Minimal,
    Aggressive,
}

impl FilterLevel {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "minimal" => Self::Minimal,
            "aggressive" => Self::Aggressive,
            _ => Self::None,
        }
    }
}

struct CommentPatterns {
    line: Option<&'static str>,
    block_start: Option<&'static str>,
    block_end: Option<&'static str>,
    doc_line: Option<&'static str>,
    doc_block_start: Option<&'static str>,
}

impl Language {
    pub fn from_path(path: &Path) -> Self {
        path.extension()
            .and_then(|e| e.to_str())
            .map(Self::from_ext)
            .unwrap_or(Self::Unknown)
    }

    pub fn from_ext(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "rs" => Self::Rust,
            "py" | "pyw" => Self::Python,
            "js" | "mjs" | "cjs" => Self::JavaScript,
            "ts" | "tsx" => Self::TypeScript,
            "go" => Self::Go,
            "c" | "h" => Self::C,
            "cpp" | "cc" | "cxx" | "hpp" | "hh" => Self::Cpp,
            "java" => Self::Java,
            "rb" => Self::Ruby,
            "sh" | "bash" | "zsh" => Self::Shell,
            "json" | "jsonc" | "json5" | "yaml" | "yml" | "toml" | "xml" | "csv" | "tsv"
            | "graphql" | "gql" | "sql" | "md" | "markdown" | "txt" | "env" | "lock" => Self::Data,
            _ => Self::Unknown,
        }
    }

    fn comments(&self) -> CommentPatterns {
        match self {
            Self::Rust => CommentPatterns {
                line: Some("//"),
                block_start: Some("/*"),
                block_end: Some("*/"),
                doc_line: Some("///"),
                doc_block_start: Some("/**"),
            },
            Self::Python => CommentPatterns {
                line: Some("#"),
                block_start: Some("\"\"\""),
                block_end: Some("\"\"\""),
                doc_line: None,
                doc_block_start: Some("\"\"\""),
            },
            Self::JavaScript | Self::TypeScript | Self::Go | Self::C | Self::Cpp | Self::Java => {
                CommentPatterns {
                    line: Some("//"),
                    block_start: Some("/*"),
                    block_end: Some("*/"),
                    doc_line: None,
                    doc_block_start: Some("/**"),
                }
            }
            Self::Ruby => CommentPatterns {
                line: Some("#"),
                block_start: Some("=begin"),
                block_end: Some("=end"),
                doc_line: None,
                doc_block_start: None,
            },
            Self::Shell => CommentPatterns {
                line: Some("#"),
                block_start: None,
                block_end: None,
                doc_line: None,
                doc_block_start: None,
            },
            Self::Data | Self::Unknown => CommentPatterns {
                line: None,
                block_start: None,
                block_end: None,
                doc_line: None,
                doc_block_start: None,
            },
        }
    }
}

lazy_static! {
    static ref MULTI_BLANK: Regex = Regex::new(r"\n{3,}").unwrap();
    static ref IMPORT_PAT: Regex = Regex::new(r"^(use |import |from |require\(|#include)").unwrap();
    static ref FUNC_SIG: Regex = Regex::new(
        r"^(pub\s+)?(async\s+)?(fn|def|function|func|class|struct|enum|trait|interface|type)\s+\w+"
    )
    .unwrap();
}

pub fn apply(content: &str, lang: Language, level: FilterLevel) -> String {
    match level {
        FilterLevel::None => content.to_string(),
        FilterLevel::Minimal => minimal(content, lang),
        FilterLevel::Aggressive => aggressive(content, lang),
    }
}

fn minimal(content: &str, lang: Language) -> String {
    if lang == Language::Data {
        return content.to_string();
    }

    let p = lang.comments();
    let mut out = String::with_capacity(content.len());
    let mut in_block = false;
    let mut in_docstring = false;

    for line in content.lines() {
        let t = line.trim();

        // Block comment tracking
        if let (Some(start), Some(end)) = (p.block_start, p.block_end) {
            if !in_docstring
                && t.contains(start)
                && !t.starts_with(p.doc_block_start.unwrap_or("###"))
            {
                in_block = true;
            }
            if in_block {
                if t.contains(end) {
                    in_block = false;
                }
                continue;
            }
        }

        // Python docstrings — keep in minimal
        if lang == Language::Python && t.starts_with("\"\"\"") {
            in_docstring = !in_docstring;
            out.push_str(line);
            out.push('\n');
            continue;
        }
        if in_docstring {
            out.push_str(line);
            out.push('\n');
            continue;
        }

        // Single-line comments (keep doc comments)
        if let Some(lc) = p.line {
            if t.starts_with(lc) {
                if p.doc_line.is_some_and(|d| t.starts_with(d)) {
                    out.push_str(line);
                    out.push('\n');
                }
                continue;
            }
        }

        out.push_str(line);
        out.push('\n');
    }

    let out = MULTI_BLANK.replace_all(&out, "\n\n");
    out.trim().to_string()
}

fn aggressive(content: &str, lang: Language) -> String {
    if lang == Language::Data {
        return minimal(content, lang);
    }

    let base = minimal(content, lang);
    let mut out = String::with_capacity(base.len() / 2);
    let mut brace_depth: i32 = 0;
    let mut in_impl = false;

    for line in base.lines() {
        let t = line.trim();

        if IMPORT_PAT.is_match(t) {
            out.push_str(line);
            out.push('\n');
            continue;
        }

        if FUNC_SIG.is_match(t) {
            out.push_str(line);
            out.push('\n');
            in_impl = true;
            brace_depth = 0;
            continue;
        }

        let opens = t.matches('{').count() as i32;
        let closes = t.matches('}').count() as i32;

        if in_impl {
            brace_depth += opens - closes;
            if brace_depth <= 1 && (t == "{" || t == "}" || t.ends_with('{')) {
                out.push_str(line);
                out.push('\n');
            }
            if brace_depth <= 0 {
                in_impl = false;
                if !t.is_empty() && t != "}" {
                    out.push_str("    // ...\n");
                }
            }
            continue;
        }

        if t.starts_with("const ")
            || t.starts_with("static ")
            || t.starts_with("pub const ")
            || t.starts_with("pub static ")
        {
            out.push_str(line);
            out.push('\n');
        }
    }

    out.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimal_strips_single_line_comments() {
        let code = "// comment\nfn main() {}\n";
        let out = apply(code, Language::Rust, FilterLevel::Minimal);
        assert!(!out.contains("// comment"));
        assert!(out.contains("fn main()"));
    }

    #[test]
    fn data_files_never_stripped() {
        let json = r#"{"packages": ["packages/*"]}"#;
        let out = apply(json, Language::Data, FilterLevel::Aggressive);
        assert!(out.contains("packages/*"));
    }

    #[test]
    fn aggressive_keeps_signatures() {
        let code = "pub fn validate(tok: &str) -> bool {\n    true\n}\n";
        let out = apply(code, Language::Rust, FilterLevel::Aggressive);
        assert!(out.contains("pub fn validate"));
    }
}
