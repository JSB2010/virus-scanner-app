# Build script for VirusTotal Scanner App
Write-Host "Building VirusTotal Scanner App for all platforms..." -ForegroundColor Green

# Build for all platforms
Write-Host "Building for all supported platforms..." -ForegroundColor Cyan
npm run tauri:build
if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed with exit code $LASTEXITCODE" -ForegroundColor Red
    exit $LASTEXITCODE
}

# Note about macOS builds
if (-not $IsMacOS) {
    Write-Host "Note: macOS bundles can only be created on a Mac system" -ForegroundColor Yellow
    Write-Host "The build process has created Windows bundles only" -ForegroundColor Yellow
}

Write-Host "Build completed successfully!" -ForegroundColor Green
Write-Host "Windows builds are available at: src-tauri/target/release/bundle/" -ForegroundColor Green
if ($IsMacOS) {
    Write-Host "macOS builds are available at: src-tauri/target/release/bundle/" -ForegroundColor Green
}
