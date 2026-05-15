# rtkmcp

Token-efficient MCP server for LLM development tools. Filters command output before it reaches your LLM context — **60-90% token savings** with zero data loss.

## Install

**macOS / Linux — one command:**
```sh
curl -sSf https://raw.githubusercontent.com/omercanga/rtkmcp/main/install.sh | sh
```

**Windows — one command (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/omercanga/rtkmcp/main/install.ps1 | iex
```

**Docker (no install required):**
```sh
docker pull ghcr.io/omercanga/rtkmcp:latest
```

**Cargo:**
```sh
cargo install rtkmcp
```

---

## Configure (pick your client)

Add **one of these** to your MCP client config — same JSON for all clients:

```json
{"mcpServers": {"rtkmcp": {"command": "rtkmcp"}}}
```

### Claude Code
File: `~/.claude/settings.json`
```json
{
  "mcpServers": {
    "rtkmcp": {
      "command": "rtkmcp"
    }
  }
}
```

### Cursor
File: `.cursor/mcp.json` (project) or `~/.cursor/mcp.json` (global)
```json
{
  "mcpServers": {
    "rtkmcp": {
      "command": "rtkmcp"
    }
  }
}
```

### Windsurf
File: `~/.codeium/windsurf/mcp_config.json`
```json
{
  "mcpServers": {
    "rtkmcp": {
      "command": "rtkmcp"
    }
  }
}
```

### VS Code (GitHub Copilot)
File: `.vscode/mcp.json`
```json
{
  "servers": {
    "rtkmcp": {
      "type": "stdio",
      "command": "rtkmcp"
    }
  }
}
```

### Docker (any client)
```json
{
  "mcpServers": {
    "rtkmcp": {
      "command": "docker",
      "args": [
        "run", "--rm", "-i",
        "-v", "${workspaceFolder}:/workspace",
        "-w", "/workspace",
        "ghcr.io/omercanga/rtkmcp:latest"
      ]
    }
  }
}
```

---

## Tools

| Tool | What it does | Savings |
|------|-------------|---------|
| `shell_run` | Run any command, strip build noise | 60-80% |
| `errors_only` | Run build/test, return ONLY failures | 70-90% |
| `read_file` | Read file, strip comments/boilerplate | 40-60% |
| `git_status` | Compact git status (porcelain) | 85% |
| `git_log` | One-liner commit history | 80% |
| `git_diff` | Condensed diff with file summaries | 60-70% |
| `list_dir` | Compact directory tree | 70% |
| `search_code` | Grep with per-file grouping | 60% |

### Examples

```
# Instead of running "cargo build" directly:
shell_run(command="cargo build", cwd="/project")

# Run tests, see only failures:
errors_only(command="cargo test", cwd="/project")

# Read a large file without comments:
read_file(path="src/auth.rs", level="minimal")

# Read just the structure (signatures only):
read_file(path="src/auth.rs", level="aggressive")

# Git status in 3 lines instead of 24:
git_status(cwd="/project")

# Last 20 commits, one line each:
git_log(limit=20)

# See what changed vs main:
git_diff(args=["main...HEAD"])

# Find all usages of a function:
search_code(pattern="fn validate_token", path="src/", file_type="rs")
```

---

## How it works

```
LLM (Claude / Cursor / Windsurf / ...)
        │  MCP tool call
        ▼
   rtkmcp server          ← single binary, no runtime deps
        │  subprocess
        ▼
  git / cargo / ...       ← native tools
        │  raw output
        ▼
   filter engine          ← strip noise, compress, group errors
        │  compact output
        ▼
   LLM context            ← 60-90% fewer tokens
```

**No hooks. No config files. No runtime dependencies.**

---

## Requirements

- **git** in PATH (for git tools)
- **rg** (ripgrep) in PATH — optional, falls back to grep for `search_code`
- Nothing else

---

## Performance

| Metric | Value |
|--------|-------|
| Binary size | ~1.8 MB |
| Startup time | < 10 ms |
| Memory usage | < 5 MB |
| Runtime deps | None |

---

## Platforms

| Platform | Binary |
|----------|--------|
| Linux x86_64 | `rtkmcp-linux-x86_64` |
| Linux ARM64 | `rtkmcp-linux-aarch64` |
| macOS Intel | `rtkmcp-macos-x86_64` |
| macOS Apple Silicon | `rtkmcp-macos-aarch64` |
| Windows x86_64 | `rtkmcp-windows-x86_64.exe` |
| Windows ARM64 | `rtkmcp-windows-aarch64.exe` |

---

## License

MIT
