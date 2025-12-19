$ErrorActionPreference = "Stop"

Write-Host "== ghx-engine wasm build proof (mesh_engine_next) =="

if (-not (Get-Command rustup -ErrorAction SilentlyContinue)) {
  throw "rustup not found. Install Rust (https://rustup.rs/) and ensure rustup is on PATH."
}

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
  throw "cargo not found. Install Rust (https://rustup.rs/) and ensure cargo is on PATH."
}

Write-Host "Ensuring wasm32 target exists..."
rustup target add wasm32-unknown-unknown | Out-Host

Write-Host "Building ghx-engine for wasm32-unknown-unknown (no default features)..."
Push-Location (Join-Path $PSScriptRoot "..")
try {
  cargo build -p ghx-engine --no-default-features --features mesh_engine_next --target wasm32-unknown-unknown | Out-Host
} finally {
  Pop-Location
}

Write-Host "OK"

