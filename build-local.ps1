# build-local.ps1 — Build a local release package for testing
# Creates: release/ironpad.exe + release/static/
# Usage: .\build-local.ps1

$ErrorActionPreference = "Stop"
$root = $PSScriptRoot

Write-Host "`n=== Ironpad Local Build ===" -ForegroundColor Cyan

# 1. Build frontend
Write-Host "`n[1/4] Building frontend..." -ForegroundColor Yellow
Push-Location "$root\frontend"
npm run build
if ($LASTEXITCODE -ne 0) { Pop-Location; throw "Frontend build failed" }
Pop-Location

# 2. Build backend (release)
Write-Host "`n[2/4] Building backend (release)..." -ForegroundColor Yellow
Push-Location "$root\backend"
cargo build --release
if ($LASTEXITCODE -ne 0) { Pop-Location; throw "Backend build failed" }
Pop-Location

# 3. Assemble release folder
Write-Host "`n[3/4] Assembling release folder..." -ForegroundColor Yellow
$releaseDir = "$root\release"
if (Test-Path $releaseDir) { Remove-Item -Recurse -Force $releaseDir }
New-Item -ItemType Directory -Path $releaseDir | Out-Null
New-Item -ItemType Directory -Path "$releaseDir\static" | Out-Null

Copy-Item "$root\backend\target\release\ironpad.exe" "$releaseDir\ironpad.exe"
Copy-Item "$root\frontend\dist\*" "$releaseDir\static\" -Recurse

# 4. Done
$size = [math]::Round((Get-Item "$releaseDir\ironpad.exe").Length / 1MB, 1)
Write-Host "`n[4/4] Done!" -ForegroundColor Green
Write-Host "  Binary:  $releaseDir\ironpad.exe ($size MB)"
Write-Host "  Static:  $releaseDir\static\"
Write-Host "`nTo run:  cd release && .\ironpad.exe" -ForegroundColor Cyan
Write-Host "(Tray icon will appear - right-click for menu)`n"
