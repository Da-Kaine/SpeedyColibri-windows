# Build script for SpeedyColibri (`coli`) on Windows (PowerShell)
#
# Usage:
#   .\scripts\build.ps1                 # CPU release build
#   $env:COLI_CUDA=1; .\scripts\build.ps1  # CUDA release build (requires nvcc / CUDA toolkit)

$ErrorActionPreference = "Stop"

Set-Location "$PSScriptRoot\.."

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Error "[build] ERROR: cargo not found. Please install Rust and ensure cargo is in your PATH."
    exit 1
}

if ($env:COLI_CUDA -eq "1") {
    Write-Host "[build] Building coli with CUDA support on Windows..."
    cargo build --release -p coli --features cuda
} else {
    Write-Host "[build] Building coli (CPU-only) on Windows..."
    cargo build --release -p coli
}

if ($LASTEXITCODE -eq 0) {
    Write-Host "[build] OK — Binary compiled successfully at target\release\coli.exe"
} else {
    Write-Error "[build] ERROR: Build failed."
    exit $LASTEXITCODE
}
