#!/usr/bin/env sh
# rtkmcp installer — macOS & Linux
# Usage: curl -sSf https://raw.githubusercontent.com/omercanga/rtkmcp/main/install.sh | sh

set -e

REPO="omercanga/rtkmcp"
BIN_NAME="rtkmcp"
INSTALL_DIR="$HOME/.local/bin"

# ── Detect OS & arch ─────────────────────────────────────────────────────────
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux)
    case "$ARCH" in
      x86_64)  ASSET="rtkmcp-linux-x86_64" ;;
      aarch64 | arm64) ASSET="rtkmcp-linux-aarch64" ;;
      *) echo "Unsupported arch: $ARCH" >&2; exit 1 ;;
    esac
    ;;
  Darwin)
    case "$ARCH" in
      x86_64)  ASSET="rtkmcp-macos-x86_64" ;;
      arm64)   ASSET="rtkmcp-macos-aarch64" ;;
      *) echo "Unsupported arch: $ARCH" >&2; exit 1 ;;
    esac
    ;;
  *)
    echo "Unsupported OS: $OS (use install.ps1 on Windows)" >&2
    exit 1
    ;;
esac

# ── Get latest version tag ───────────────────────────────────────────────────
LATEST=$(curl -sSf "https://api.github.com/repos/${REPO}/releases/latest" \
  | grep '"tag_name"' | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')

if [ -z "$LATEST" ]; then
  echo "Could not determine latest release. Check your internet connection." >&2
  exit 1
fi

echo "Installing rtkmcp ${LATEST} for ${OS}/${ARCH}..."

# ── Download ──────────────────────────────────────────────────────────────────
URL="https://github.com/${REPO}/releases/download/${LATEST}/${ASSET}"
TMP="$(mktemp)"

if command -v curl >/dev/null 2>&1; then
  curl -sSfL "$URL" -o "$TMP"
elif command -v wget >/dev/null 2>&1; then
  wget -qO "$TMP" "$URL"
else
  echo "curl or wget required" >&2; exit 1
fi

# ── Verify checksum (optional but recommended) ───────────────────────────────
SUMS_URL="https://github.com/${REPO}/releases/download/${LATEST}/SHA256SUMS.txt"
TMP_SUMS="$(mktemp)"
if curl -sSfL "$SUMS_URL" -o "$TMP_SUMS" 2>/dev/null; then
  expected=$(grep "$ASSET" "$TMP_SUMS" | awk '{print $1}')
  if [ -n "$expected" ]; then
    if command -v sha256sum >/dev/null 2>&1; then
      actual=$(sha256sum "$TMP" | awk '{print $1}')
    elif command -v shasum >/dev/null 2>&1; then
      actual=$(shasum -a 256 "$TMP" | awk '{print $1}')
    fi
    if [ -n "$actual" ] && [ "$actual" != "$expected" ]; then
      echo "Checksum mismatch! Download may be corrupted." >&2
      rm -f "$TMP" "$TMP_SUMS"
      exit 1
    fi
    echo "Checksum OK"
  fi
fi
rm -f "$TMP_SUMS"

# ── Install ───────────────────────────────────────────────────────────────────
mkdir -p "$INSTALL_DIR"
mv "$TMP" "$INSTALL_DIR/$BIN_NAME"
chmod +x "$INSTALL_DIR/$BIN_NAME"

# ── PATH hint ────────────────────────────────────────────────────────────────
echo ""
echo "rtkmcp installed to: $INSTALL_DIR/$BIN_NAME"
echo ""

# Check if INSTALL_DIR is already in PATH
case ":$PATH:" in
  *":$INSTALL_DIR:"*)
    echo "Already in PATH. Run: rtkmcp --version"
    ;;
  *)
    echo "Add to PATH (add to your ~/.bashrc, ~/.zshrc, or ~/.profile):"
    echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
    echo ""
    echo "Or run now:"
    echo "  export PATH=\"\$HOME/.local/bin:\$PATH\" && rtkmcp --version"
    ;;
esac

echo ""
echo "Configure your MCP client (copy one of these):"
echo ""
echo "Claude Code (~/.claude/settings.json):"
echo '  {"mcpServers": {"rtkmcp": {"command": "rtkmcp"}}}'
echo ""
echo "Cursor (.cursor/mcp.json) / Windsurf (~/.codeium/windsurf/mcp_config.json):"
echo '  {"mcpServers": {"rtkmcp": {"command": "rtkmcp"}}}'
