# SPDX-License-Identifier: GPL-3.0-only
# Records the USB traffic between a camera and its maker's app on Windows, for the clean-room
# protocol notes in docs/cameras/ (ADR-0026). Guide: docs/cameras/capture-guide.md.
#
# Run in PowerShell **as Administrator**, from the repository root:
#   powershell -ExecutionPolicy Bypass -File scripts\camera-capture.ps1 -Name a6400
# Stop with Ctrl+C. The file goes to captures\ (git ignores it: never commit a capture).

param(
    [string]$Name = "camera"
)

$ErrorActionPreference = "Stop"

$dumpcap = @(
    "$env:ProgramFiles\Wireshark\dumpcap.exe",
    "${env:ProgramFiles(x86)}\Wireshark\dumpcap.exe"
) | Where-Object { Test-Path $_ } | Select-Object -First 1

if (-not $dumpcap) {
    Write-Host "Wireshark is not installed. Install it from https://www.wireshark.org/download.html"
    Write-Host "and tick 'USBPcap' during the install, then restart Windows."
    exit 1
}

# Lines look like: 5. \\.\USBPcap1 (USBPcap1)
$interfaces = & $dumpcap -D 2>$null |
    Where-Object { $_ -match "USBPcap" } |
    ForEach-Object { ($_ -split "\s+")[1] }

if (-not $interfaces) {
    Write-Host "No USBPcap interface found. Reinstall Wireshark with 'USBPcap' ticked,"
    Write-Host "restart Windows, and run this window as Administrator."
    exit 1
}

New-Item -ItemType Directory -Force -Path "captures" | Out-Null
$stamp = Get-Date -Format "yyyyMMdd-HHmmss"
$out = Join-Path "captures" "$Name-$stamp.pcapng"

$arguments = @()
foreach ($i in $interfaces) { $arguments += @("-i", $i) }
$arguments += @("-w", $out)

Write-Host ""
Write-Host "Recording all USB buses ($($interfaces -join ', ')) to $out"
Write-Host "Now, in the maker's app: connect, live view ~5 s, shoot a JPEG, switch to RAW and"
Write-Host "shoot one, change ISO once, disconnect. Then press Ctrl+C here."
Write-Host ""

try {
    & $dumpcap @arguments
}
finally {
    if (Test-Path $out) {
        $mb = [math]::Round((Get-Item $out).Length / 1MB, 1)
        Write-Host ""
        Write-Host "Saved $out ($mb MB). Copy it to the Mac (USB stick or cloud); do not commit it."
    }
}
