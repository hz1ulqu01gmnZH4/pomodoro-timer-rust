#!/bin/bash

# Build script for cross-compiling to Windows from Linux/WSL
# Requires: mingw-w64 package installed (sudo apt-get install mingw-w64)

echo "Building Rust Pomodoro Timer for Windows..."

# Check if mingw is installed
if ! command -v x86_64-w64-mingw32-gcc &> /dev/null; then
    echo "Error: mingw-w64 is not installed."
    echo "Please run: sudo apt-get install mingw-w64"
    exit 1
fi

# Add Windows target if not already added
rustup target add x86_64-pc-windows-gnu 2>/dev/null

# Build for Windows
echo "Cross-compiling for Windows x86_64..."
cargo build --release --target x86_64-pc-windows-gnu

if [ $? -eq 0 ]; then
    echo "Build successful!"
    echo "Binary location: target/x86_64-pc-windows-gnu/release/rust_pomodoro.exe"
    
    # Copy to convenient location
    cp target/x86_64-pc-windows-gnu/release/rust_pomodoro.exe ./rust_pomodoro_windows.exe 2>/dev/null
    
    echo "Executable copied to: ./rust_pomodoro_windows.exe"
else
    echo "Build failed. Please check the error messages above."
    exit 1
fi