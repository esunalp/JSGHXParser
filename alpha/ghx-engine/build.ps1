# Build script for ghx-engine WASM
# Usage: .\build.ps1 [release]

param(
    [switch]$release
)

Write-Host "Building ghx-engine WASM..." -ForegroundColor Cyan

if ($release) {
    Write-Host "Building in RELEASE mode (optimized)" -ForegroundColor Green
    wasm-pack build --target web --out-dir ..\web\pkg
} else {
    Write-Host "Building in DEBUG mode (with debug_logs)" -ForegroundColor Yellow
    wasm-pack build --target web --out-dir ..\web\pkg --no-typescript --no-opt -- --features "debug_logs"
}

# Remove the .gitignore that wasm-pack creates (so pkg files can be committed to git)
$gitignorePath = "..\web\pkg\.gitignore"
if (Test-Path $gitignorePath) {
    Remove-Item $gitignorePath
    Write-Host "Removed pkg/.gitignore (files can now be committed to git)" -ForegroundColor Green
}

Write-Host "Build complete! Output at: ..\web\pkg" -ForegroundColor Cyan
