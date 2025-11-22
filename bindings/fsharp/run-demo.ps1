#!/usr/bin/env pwsh
Set-StrictMode -Version Latest

Param(
    [switch]$Release
)

# Build native library
Write-Host "Building rrpc_core (release)..."
Set-Location "$(Resolve-Path ..\..\)"
if ($Release) { cargo build --release } else { cargo build }

# Build F# app
Set-Location "$PSScriptRoot"
dotnet build

# Copy native dll
$native = if ($Release) { "..\..\target\release\rrpc_core.dll" } else { "..\..\target\debug\rrpc_core.dll" }
$dest = Join-Path $PSScriptRoot "bin\Debug\net9.0\rrpc_core.dll"
Copy-Item -Path $native -Destination $dest -Force

Write-Host "Running F# demo (press Enter to finish)"
dotnet run
