# deploy-lambda.ps1
# ─────────────────────────────────────────────────────────────────────────────
# Deploy the Notes App backend to AWS Lambda via SAM CLI.
#
# Prerequisites:
#   1. AWS CLI configured with credentials: aws configure
#   2. SAM CLI installed: https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/install-sam-cli.html
#   3. cargo-lambda installed: cargo install cargo-lambda
#
# Usage:
#   .\deploy-lambda.ps1                    # Guided first-time deploy
#   .\deploy-lambda.ps1 -SkipBuild        # Deploy without rebuilding
#   .\deploy-lambda.ps1 -Stage staging    # Deploy to staging environment
# ─────────────────────────────────────────────────────────────────────────────

param(
    [ValidateSet("dev", "staging", "prod")]
    [string]$Stage = "prod",

    [switch]$SkipBuild,
    [switch]$Guided
)

$ErrorActionPreference = "Stop"

Write-Host ""
Write-Host "════════════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "  Notes App Backend — Lambda Deployment ($Stage)" -ForegroundColor Cyan
Write-Host "════════════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host ""

# ── Step 1: Verify prerequisites ─────────────────────────────────────────
Write-Host "[1/4] Checking prerequisites..." -ForegroundColor Yellow

try {
    $awsVersion = aws --version 2>&1
    Write-Host "  AWS CLI: $awsVersion" -ForegroundColor Green
} catch {
    Write-Host "  ERROR: AWS CLI not found. Install from https://aws.amazon.com/cli/" -ForegroundColor Red
    exit 1
}

try {
    $samVersion = sam --version 2>&1
    Write-Host "  SAM CLI: $samVersion" -ForegroundColor Green
} catch {
    Write-Host "  ERROR: SAM CLI not found. Install from https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/install-sam-cli.html" -ForegroundColor Red
    exit 1
}

# ── Step 2: Build ─────────────────────────────────────────────────────────
if (-not $SkipBuild) {
    Write-Host ""
    Write-Host "[2/4] Building Lambda binary..." -ForegroundColor Yellow
    & "$PSScriptRoot\build-lambda.ps1" -Profile release
    if ($LASTEXITCODE -ne 0) {
        Write-Host "  Build failed. Aborting deployment." -ForegroundColor Red
        exit 1
    }
} else {
    Write-Host ""
    Write-Host "[2/4] Skipping build (using existing binary)." -ForegroundColor Yellow
}

# ── Step 3: SAM Build ────────────────────────────────────────────────────
Write-Host ""
Write-Host "[3/4] Running SAM build..." -ForegroundColor Yellow
sam build --beta-features

if ($LASTEXITCODE -ne 0) {
    Write-Host "  SAM build failed." -ForegroundColor Red
    exit 1
}

# ── Step 4: SAM Deploy ───────────────────────────────────────────────────
Write-Host ""
Write-Host "[4/4] Deploying to AWS ($Stage)..." -ForegroundColor Yellow

$deployArgs = @("deploy")

# First deploy or explicit guided mode
$samConfigExists = Test-Path "samconfig.toml"
if ($Guided -or -not $samConfigExists) {
    $deployArgs += "--guided"
    Write-Host "  Running in guided mode (first-time deployment or -Guided flag set)." -ForegroundColor Yellow
    Write-Host "  SAM will prompt you for parameters (DatabaseUrl, JwtSecret, VPC config, etc.)." -ForegroundColor Yellow
} else {
    Write-Host "  Using existing samconfig.toml configuration." -ForegroundColor Green
}

$deployArgs += "--stack-name"
$deployArgs += "notes-backend-$Stage"
$deployArgs += "--parameter-overrides"
$deployArgs += "Stage=$Stage"
$deployArgs += "--capabilities"
$deployArgs += "CAPABILITY_IAM"
$deployArgs += "--no-confirm-changeset"

& sam @deployArgs

if ($LASTEXITCODE -ne 0) {
    Write-Host ""
    Write-Host "  DEPLOYMENT FAILED" -ForegroundColor Red
    exit 1
}

# ── Done ──────────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "════════════════════════════════════════════════════════════════" -ForegroundColor Green
Write-Host "  Deployment to $Stage SUCCEEDED" -ForegroundColor Green
Write-Host "════════════════════════════════════════════════════════════════" -ForegroundColor Green
Write-Host ""

# Print the API endpoint
Write-Host "  API Endpoint:" -ForegroundColor Cyan
aws cloudformation describe-stacks `
    --stack-name "notes-backend-$Stage" `
    --query "Stacks[0].Outputs[?OutputKey=='ApiEndpoint'].OutputValue" `
    --output text

Write-Host ""
