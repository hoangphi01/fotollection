# Set up a fresh Windows machine to build Fotollection. See docs/DEVELOPMENT.md §1.
#
# Usage: powershell -ExecutionPolicy Bypass -File scripts\bootstrap.ps1 [-Install] [-Tools]
#   -Install  install missing prerequisites with winget
#   -Tools    also install optional dev tools (cargo-sweep, cargo-deny)
# Versions come from the repo: rust-toolchain.toml, .node-version, Cargo.lock, package-lock.json.
param([switch]$Install, [switch]$Tools)
$ErrorActionPreference = 'Stop'
Set-Location (Join-Path $PSScriptRoot '..')

$missing = $false
function Ok($msg) { Write-Host "  ✓ $msg" }
function Need($msg) { Write-Host "  ✗ $msg"; $script:missing = $true }
function Has($cmd) { [bool](Get-Command $cmd -ErrorAction SilentlyContinue) }
function WingetInstall($id) {
  winget install --id $id --exact --accept-source-agreements --accept-package-agreements --silent
}

Write-Host '• System prerequisites'
if (Has git) { Ok 'git' } elseif ($Install) { WingetInstall 'Git.Git'; Ok 'git (installed)' } else { Need 'git' }

$vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
$hasMsvc = (Test-Path $vswhere) -and (& $vswhere -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath)
if ($hasMsvc) { Ok 'MSVC Build Tools' }
elseif ($Install) {
  winget install --id Microsoft.VisualStudio.2022.BuildTools --exact --silent --accept-source-agreements --accept-package-agreements `
    --override '--wait --quiet --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended'
  Ok 'MSVC Build Tools (installed)'
} else { Need 'MSVC Build Tools (Desktop development with C++)' }

$webview = Get-ItemProperty 'HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}' -ErrorAction SilentlyContinue
if ($webview) { Ok "WebView2 $($webview.pv)" } elseif ($Install) { WingetInstall 'Microsoft.EdgeWebView2Runtime'; Ok 'WebView2 (installed)' } else { Need 'WebView2 runtime' }

Write-Host '• Rust'
if (-not (Has rustup)) {
  if ($Install) { WingetInstall 'Rustlang.Rustup'; $env:Path += ";$env:USERPROFILE\.cargo\bin" } else { Need 'rustup (https://rustup.rs)' }
}
if (Has rustup) {
  rustup show active-toolchain *> $null
  if ($LASTEXITCODE -ne 0) { rustup toolchain install }   # reads rust-toolchain.toml
  Ok (rustc --version)
}

Write-Host '• Node.js'
$nodeTarget = (Get-Content .node-version).Trim()
if (Has node) {
  $major = [int](node -p 'process.versions.node.split(".")[0]')
  if ($major -ge 22) { Ok "node $(node --version) (recommended: $nodeTarget LTS)" } else { Need "node >= 22 (found $(node --version))" }
} elseif ($Install) { WingetInstall 'OpenJS.NodeJS.LTS'; Ok 'node (installed; restart the shell)' } else { Need "node >= 22 (recommended $nodeTarget LTS)" }

if ($missing) { Write-Host 'bootstrap: prerequisites missing (see ✗ above)'; exit 1 }

Write-Host '• Project dependencies (from lockfiles)'
npm ci
cargo fetch --locked

if ($Tools) {
  Write-Host '• Optional dev tools'
  cargo install --locked cargo-sweep cargo-deny
}

Write-Host "bootstrap: done — run 'npm run dev' to start the app"
