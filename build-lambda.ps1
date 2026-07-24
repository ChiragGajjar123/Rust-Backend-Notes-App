# build-lambda.ps1
# ─────────────────────────────────────────────────────────────────────────────
# Build the Notes App backend for AWS Lambda deployment.
#
# Prerequisites:
#   1. Install cargo-lambda: cargo install cargo-lambda
#      (or via pip: pip3 install cargo-lambda)
#   2. Ensure Rust toolchain is installed with the stable channel.
#
# Usage:
#   .\build-lambda.ps1                 # Default release build
#   .\build-lambda.ps1 -Profile debug  # Debug build for testing
# ─────────────────────────────────────────────────────────────────────────────

param(
    [ValidateSet("release", "debug")]
    [string]$Profile = "release"
)

$ErrorActionPreference = "Stop"

Write-Host ""
Write-Host "════════════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "  Notes App Backend — Lambda Build" -ForegroundColor Cyan
Write-Host "════════════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host ""

# ── Step 1: Verify cargo-lambda is installed ──────────────────────────────
Write-Host "[1/3] Checking cargo-lambda installation..." -ForegroundColor Yellow
try {
    $cargoLambdaVersion = cargo lambda --version 2>&1
    Write-Host "  Found: $cargoLambdaVersion" -ForegroundColor Green
} catch {
    Write-Host "  ERROR: cargo-lambda is not installed." -ForegroundColor Red
    Write-Host "  Install it with: cargo install cargo-lambda" -ForegroundColor Red
    Write-Host "  Or via pip:      pip3 install cargo-lambda" -ForegroundColor Red
    exit 1
}

# ── Step 2: Build for Lambda ──────────────────────────────────────────────
Write-Host ""
Write-Host "[2/3] Building for AWS Lambda (x86_64, feature: lambda)..." -ForegroundColor Yellow
Write-Host ""

$buildArgs = @("lambda", "build", "--features", "lambda", "--no-default-features")

if ($Profile -eq "release") {
    $buildArgs += "--release"
    Write-Host "  Profile: release (optimized)" -ForegroundColor Green
} else {
    Write-Host "  Profile: debug (fast compile)" -ForegroundColor Yellow
}

# cargo-lambda automatically targets the correct Linux architecture
& cargo @buildArgs

if ($LASTEXITCODE -ne 0) {
    Write-Host ""
    Write-Host "  BUILD FAILED" -ForegroundColor Red
    exit 1
}

# ── Step 3: Report output ────────────────────────────────────────────────
Write-Host ""
Write-Host "[3/3] Build complete!" -ForegroundColor Green
Write-Host ""

$outputDir = "target\lambda\notes_backend"
if (Test-Path $outputDir) {
    $bootstrapPath = Join-Path $outputDir "bootstrap"
    if (Test-Path $bootstrapPath) {
        $size = (Get-Item $bootstrapPath).Length / 1MB
        Write-Host "  Output: $bootstrapPath" -ForegroundColor Cyan
        Write-Host "  Size:   $([math]::Round($size, 2)) MB" -ForegroundColor Cyan
    }
}

Write-Host ""
Write-Host "════════════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "  Next step: Run .\deploy-lambda.ps1 to deploy to AWS" -ForegroundColor Cyan
Write-Host "════════════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host ""
