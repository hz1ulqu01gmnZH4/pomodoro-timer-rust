# PowerShell build script for Windows
# Run with: powershell -ExecutionPolicy Bypass -File build-windows.ps1

Write-Host "Building Rust Pomodoro Timer for Windows..." -ForegroundColor Green

# Check if Rust is installed
if (!(Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "Error: Rust is not installed. Please install from https://rustup.rs/" -ForegroundColor Red
    exit 1
}

# Add Windows target if not already added
Write-Host "Ensuring Windows target is installed..." -ForegroundColor Yellow
rustup target add x86_64-pc-windows-msvc 2>$null

# Clean previous builds
Write-Host "Cleaning previous builds..." -ForegroundColor Yellow
cargo clean 2>$null

# Build for Windows
Write-Host "Building release version..." -ForegroundColor Yellow
cargo build --release --target x86_64-pc-windows-msvc

if ($LASTEXITCODE -eq 0) {
    Write-Host "Build successful!" -ForegroundColor Green
    Write-Host "Binary location: target\x86_64-pc-windows-msvc\release\rust_pomodoro.exe" -ForegroundColor Cyan
    
    # Copy to convenient location
    Copy-Item "target\x86_64-pc-windows-msvc\release\rust_pomodoro.exe" -Destination ".\rust_pomodoro.exe" -Force
    Write-Host "Executable copied to: .\rust_pomodoro.exe" -ForegroundColor Cyan
    
    Write-Host "`nYou can now run the application with: .\rust_pomodoro.exe" -ForegroundColor Green
} else {
    Write-Host "Build failed. Please check the error messages above." -ForegroundColor Red
    exit 1
}