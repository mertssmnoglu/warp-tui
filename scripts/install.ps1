#!/usr/bin/env pwsh
# Install script for warp-tui (Windows)
#
# Downloads the latest release binary for the current architecture from
# GitHub releases and installs it into $InstallDir (defaults to
# "$env:LOCALAPPDATA\warp-tui\bin").
#
# Usage:
#   irm https://raw.githubusercontent.com/mertssmnoglu/warp-tui/main/scripts/install.ps1 | iex
#
# To customize, set environment variables before running:
#   $env:INSTALL_DIR = "C:\tools\warp-tui"
#   $env:VERSION = "v1.2.3"
#   irm https://raw.githubusercontent.com/mertssmnoglu/warp-tui/main/scripts/install.ps1 | iex

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$Repo = "mertssmnoglu/warp-tui"
$BinName = "warp-tui"
$InstallDir = if ($env:INSTALL_DIR) { $env:INSTALL_DIR } else { Join-Path $env:LOCALAPPDATA "warp-tui\bin" }
$Version = if ($env:VERSION) { $env:VERSION } else { "latest" }

function Write-Log([string]$Message) {
    Write-Host "==> $Message"
}

function Get-Arch {
    switch ([System.Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture) {
        "X64" { return "x86_64" }
        "Arm64" { return "aarch64" }
        default { throw "unsupported architecture: $([System.Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture)" }
    }
}

function Resolve-Version {
    $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest" -Headers @{ "User-Agent" = "warp-tui-install-script" }
    return $release.tag_name
}

$arch = Get-Arch

$tag = $Version
if ($tag -eq "latest") {
    $tag = Resolve-Version
    if (-not $tag) { throw "failed to resolve the latest release tag" }
}
$versionNum = $tag -replace '^v', ''

$target = "$arch-pc-windows-msvc"
$asset = "$BinName-$versionNum-$target.exe"
$url = "https://github.com/$Repo/releases/download/$tag/$asset"

$tmpDir = Join-Path ([System.IO.Path]::GetTempPath()) ([System.Guid]::NewGuid().ToString())
New-Item -ItemType Directory -Path $tmpDir | Out-Null
$tmpFile = Join-Path $tmpDir $asset

try {
    Write-Log "Downloading $asset ($tag)"
    Invoke-WebRequest -Uri $url -OutFile $tmpFile -UseBasicParsing

    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    $destination = Join-Path $InstallDir "$BinName.exe"
    Copy-Item -Path $tmpFile -Destination $destination -Force

    Write-Log "Installed $BinName to $destination"

    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($userPath -notlike "*$InstallDir*") {
        $newPath = "$userPath;$InstallDir"
        [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
        $env:Path = "$env:Path;$InstallDir"
        Write-Log "Added $InstallDir to your user PATH. Restart your shell for it to take effect in new windows."
    }
}
finally {
    Remove-Item -Path $tmpDir -Recurse -Force -ErrorAction SilentlyContinue
}
