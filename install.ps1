# install.ps1 — one-stop installer for run-bob on Windows.
#
# Usage:
#   iwr -useb https://raw.githubusercontent.com/amwtke/run-bob/master/install.ps1 | iex
#
# Environment:
#   $env:RUN_BOB_INSTALL_DIR  destination directory (default: $env:USERPROFILE\bin)
#   $env:RUN_BOB_VERSION      release tag to install  (default: latest)
#
# Downloads the matching prebuilt zip from GitHub Releases and drops
# `run-bob.exe` into the install dir.

$ErrorActionPreference = 'Stop'

$Repo     = 'amwtke/run-bob'
$BinName  = 'run-bob'
$InstallDir = if ($env:RUN_BOB_INSTALL_DIR) { $env:RUN_BOB_INSTALL_DIR } else { Join-Path $env:USERPROFILE 'bin' }
$Version  = if ($env:RUN_BOB_VERSION)      { $env:RUN_BOB_VERSION }      else { 'latest' }

function Write-Info  ($msg) { Write-Host "→ $msg"  -ForegroundColor Blue }
function Write-Ok    ($msg) { Write-Host "✓ $msg"  -ForegroundColor Green }
function Write-Warn2 ($msg) { Write-Host "⚠ $msg"  -ForegroundColor Yellow }
function Die         ($msg) { Write-Host "error: $msg" -ForegroundColor Red; exit 1 }

# --- detect arch (Windows 64-bit only for now) ---
if (-not [Environment]::Is64BitOperatingSystem) {
    Die 'only 64-bit Windows is supported. See https://github.com/amwtke/run-bob/releases for manual download.'
}
$target = 'x86_64-pc-windows-msvc'

# --- resolve latest version ---
if ($Version -eq 'latest') {
    Write-Info 'resolving latest release...'
    try {
        $api = Invoke-RestMethod -UseBasicParsing "https://api.github.com/repos/$Repo/releases/latest"
    } catch {
        Die "failed to query GitHub API: $_"
    }
    $Version = $api.tag_name
    if (-not $Version) { Die 'could not resolve latest version. Try setting $env:RUN_BOB_VERSION.' }
}

$asset = "$BinName-$Version-$target.zip"
$url   = "https://github.com/$Repo/releases/download/$Version/$asset"

Write-Host ''
Write-Host 'run-bob installer'
Write-Host "  target  : $target"
Write-Host "  version : $Version"
Write-Host "  url     : $url"
Write-Host "  destdir : $InstallDir"
Write-Host ''

$tmp = Join-Path $env:TEMP "run-bob-install-$([guid]::NewGuid())"
New-Item -ItemType Directory -Path $tmp -Force | Out-Null
try {
    $archive = Join-Path $tmp $asset

    Write-Info 'downloading...'
    try {
        Invoke-WebRequest -UseBasicParsing -Uri $url -OutFile $archive
    } catch {
        Die "download failed: $url`n$_"
    }

    Write-Info 'extracting...'
    Expand-Archive -Path $archive -DestinationPath $tmp -Force

    $extracted = Join-Path $tmp "$BinName-$Version-$target"
    $binSrc = Join-Path $extracted "$BinName.exe"
    if (-not (Test-Path $binSrc)) { Die "binary not found in archive: $binSrc" }

    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    $binDst = Join-Path $InstallDir "$BinName.exe"
    Copy-Item $binSrc $binDst -Force

    Write-Host ''
    Write-Ok "installed: $binDst"
    Write-Host ''

    # --- PATH guidance ---
    $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    $entries  = if ($userPath) { $userPath -split ';' } else { @() }
    if ($entries -contains $InstallDir) {
        Write-Ok "$InstallDir is on your user PATH."
        Write-Host '  Try: run-bob --version'
    } else {
        Write-Warn2 "$InstallDir is NOT on your user PATH."
        Write-Host ''
        Write-Host '  Add it permanently (PowerShell, current user):'
        Write-Host ''
        Write-Host "    [Environment]::SetEnvironmentVariable('Path', `"$InstallDir;`" + [Environment]::GetEnvironmentVariable('Path', 'User'), 'User')"
        Write-Host ''
        Write-Host '  Then open a new shell.'
    }
} finally {
    Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
}
