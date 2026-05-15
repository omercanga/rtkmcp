use crate::protocol::types::Tool;
use serde_json::json;

pub fn all_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "shell_run",
            description: "Run any shell command and return filtered output. Removes build noise \
                          (Compiling, Downloading, progress bars) while keeping errors and results. \
                          Use this instead of Bash for 60-80% token savings.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "Shell command to run (e.g. 'cargo build', 'npm install')"
                    },
                    "cwd": {
                        "type": "string",
                        "description": "Working directory (default: current directory)"
                    },
                    "filter": {
                        "type": "string",
                        "enum": ["auto", "none"],
                        "description": "Filter level: 'auto' (default) removes noise, 'none' returns raw output"
                    }
                },
                "required": ["command"]
            }),
        },
        Tool {
            name: "errors_only",
            description: "Run a build or test command and return ONLY errors and failures. \
                          Passing tests are dropped. Achieves 70-90% token savings. \
                          Supports cargo, npm, go, dotnet, and generic commands.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "Build/test command (e.g. 'cargo test', 'npm test', 'go test ./...')"
                    },
                    "cwd": {
                        "type": "string",
                        "description": "Working directory"
                    }
                },
                "required": ["command"]
            }),
        },
        Tool {
            name: "read_file",
            description: "Read a file with intelligent filtering. Strips comments and boilerplate \
                          (40-60% savings). Use level='aggressive' for just signatures/structure.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "File path to read"
                    },
                    "level": {
                        "type": "string",
                        "enum": ["none", "minimal", "aggressive"],
                        "description": "'none' = raw, 'minimal' = strip comments (default), 'aggressive' = signatures only"
                    },
                    "max_lines": {
                        "type": "integer",
                        "description": "Maximum lines to return (default: 500)"
                    }
                },
                "required": ["path"]
            }),
        },
        Tool {
            name: "git_status",
            description: "Compact git status. 85% token savings over plain 'git status'. \
                          Shows branch + changed files in porcelain format.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "cwd": {
                        "type": "string",
                        "description": "Repository path (default: current directory)"
                    }
                }
            }),
        },
        Tool {
            name: "git_log",
            description: "Git commit history, one line per commit. 80% token savings. \
                          Each commit: hash + subject + date + author.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "limit": {
                        "type": "integer",
                        "description": "Number of commits to show (default: 20)"
                    },
                    "cwd": {
                        "type": "string",
                        "description": "Repository path"
                    },
                    "args": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Extra git log arguments (e.g. ['--author=Alice', 'main..feature'])"
                    }
                }
            }),
        },
        Tool {
            name: "git_diff",
            description: "Condensed git diff. Shows file headers, hunk headers, +/- lines. \
                          Context lines reduced. 60-70% token savings.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "args": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Git diff arguments (e.g. ['HEAD~1'], ['main...feature'])"
                    },
                    "cwd": {
                        "type": "string",
                        "description": "Repository path"
                    },
                    "max_lines": {
                        "type": "integer",
                        "description": "Max output lines (default: 300)"
                    }
                }
            }),
        },
        Tool {
            name: "list_dir",
            description: "Compact directory listing. Groups files by type, shows sizes. \
                          70% token savings over 'ls -la'.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Directory path (default: current directory)"
                    },
                    "depth": {
                        "type": "integer",
                        "description": "Tree depth (default: 2, max: 4)"
                    }
                }
            }),
        },
        Tool {
            name: "search_code",
            description: "Search for a pattern in code files. Results grouped by file, \
                          with line numbers. 60% token savings over plain grep.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "Search pattern (regex supported)"
                    },
                    "path": {
                        "type": "string",
                        "description": "Directory or file to search (default: current directory)"
                    },
                    "file_type": {
                        "type": "string",
                        "description": "File extension filter (e.g. 'rs', 'ts', 'py')"
                    },
                    "max_results": {
                        "type": "integer",
                        "description": "Maximum results to return (default: 50)"
                    }
                },
                "required": ["pattern"]
            }),
        },
    ]
}
