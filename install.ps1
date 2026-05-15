# rtkmcp installer — Windows (PowerShell 5.1+)
# Usage: irm https://raw.githubusercontent.com/omercanga/rtkmcp/main/install.ps1 | iex

$ErrorActionPreference = 'Stop'

$Repo      = "omercanga/rtkmcp"
$BinName   = "rtkmcp.exe"
$InstallDir = "$env:LOCALAPPDATA\rtkmcp"

# ── Detect arch ───────────────────────────────────────────────────────────────
$Arch = if ([System.Environment]::Is64BitOperatingSystem) {
    if ($env:PROCESSOR_ARCHITECTURE -eq 'ARM64') { "aarch64" } else { "x86_64" }
} else {
    Write-Error "32-bit Windows is not supported."
    exit 1
}

$Asset = "rtkmcp-windows-${Arch}.exe"

# ── Get latest release tag ───────────────────────────────────────────────────
Write-Host "Fetching latest rtkmcp release..." -ForegroundColor Cyan
$Release = Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/latest"
$Version = $Release.tag_name

if (-not $Version) {
    Write-Error "Could not determine latest release. Check your internet connection."
    exit 1
}

Write-Host "Installing rtkmcp $Version for Windows/$Arch..." -ForegroundColor Cyan

# ── Download ──────────────────────────────────────────────────────────────────
$Url    = "https://github.com/$Repo/releases/download/$Version/$Asset"
$TmpPath = Join-Path $env:TEMP $Asset

Write-Host "Downloading $Asset..."
Invoke-WebRequest -Uri $Url -OutFile $TmpPath -UseBasicParsing

# ── Verify checksum ───────────────────────────────────────────────────────────
$SumsUrl = "https://github.com/$Repo/releases/download/$Version/SHA256SUMS.txt"
try {
    $Sums = Invoke-WebRequest -Uri $SumsUrl -UseBasicParsing -ErrorAction SilentlyContinue
    if ($Sums -and $Sums.Content) {
        $Expected = ($Sums.Content -split "`n" | Where-Object { $_ -match $Asset }) -replace '\s+.*', '' -replace '^\s+', ''
        if ($Expected) {
            $Actual = (Get-FileHash -Path $TmpPath -Algorithm SHA256).Hash.ToLower()
            if ($Actual -ne $Expected.ToLower()) {
                Write-Error "Checksum mismatch! Download may be corrupted."
                Remove-Item $TmpPath -Force
                exit 1
            }
            Write-Host "Checksum OK" -ForegroundColor Green
        }
    }
} catch {
    # Checksum fetch failed — continue without verification
}

# ── Install ───────────────────────────────────────────────────────────────────
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir | Out-Null
}

$Destination = Join-Path $InstallDir $BinName
Copy-Item -Path $TmpPath -Destination $Destination -Force
Remove-Item $TmpPath -Force

Write-Host ""
Write-Host "rtkmcp installed to: $Destination" -ForegroundColor Green

# ── Add to user PATH ──────────────────────────────────────────────────────────
$UserPath = [System.Environment]::GetEnvironmentVariable("PATH", "User")
if ($UserPath -notlike "*$InstallDir*") {
    [System.Environment]::SetEnvironmentVariable(
        "PATH", "$InstallDir;$UserPath", "User"
    )
    Write-Host "Added to user PATH (restart terminal to take effect)" -ForegroundColor Yellow
    $env:PATH = "$InstallDir;$env:PATH"
} else {
    Write-Host "Already in PATH" -ForegroundColor Green
}

# ── Verify ────────────────────────────────────────────────────────────────────
Write-Host ""
try {
    $v = & $Destination --version 2>&1
    Write-Host "Verified: $v" -ForegroundColor Green
} catch {
    Write-Host "Binary installed but could not verify. Try: rtkmcp --version" -ForegroundColor Yellow
}

# ── Config hint ───────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "Configure your MCP client:" -ForegroundColor Cyan
Write-Host ""
Write-Host 'Claude Code  (~/.claude/settings.json):'
Write-Host '  {"mcpServers": {"rtkmcp": {"command": "rtkmcp"}}}'
Write-Host ""
Write-Host 'Cursor       (.cursor/mcp.json):'
Write-Host '  {"mcpServers": {"rtkmcp": {"command": "rtkmcp"}}}'
Write-Host ""
Write-Host 'Windsurf     (~/.codeium/windsurf/mcp_config.json):'
Write-Host '  {"mcpServers": {"rtkmcp": {"command": "rtkmcp"}}}'
