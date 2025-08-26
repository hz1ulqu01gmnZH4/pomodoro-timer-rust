# Building Rust Pomodoro Timer on Windows

## Prerequisites

1. **Install Rust**: Download from [https://rustup.rs/](https://rustup.rs/)
2. **Visual Studio Build Tools** (for MSVC): 
   - Download Visual Studio Build Tools from [https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022)
   - Install "Desktop development with C++" workload

## Building

### Option 1: Using PowerShell Script (Recommended)

```powershell
# In PowerShell, navigate to the rust-pomodoro directory
cd C:\path\to\rust-pomodoro

# Run the build script
powershell -ExecutionPolicy Bypass -File build-windows.ps1
```

### Option 2: Manual Build

```powershell
# Add Windows target (if not already added)
rustup target add x86_64-pc-windows-msvc

# Build the release version
cargo build --release --target x86_64-pc-windows-msvc

# The executable will be at:
# target\x86_64-pc-windows-msvc\release\rust_pomodoro.exe
```

### Option 3: Run Directly

```powershell
# Run in release mode
cargo run --release --target x86_64-pc-windows-msvc
```

## Testing the Overlay

1. Run the application
2. Click "Start" to begin the timer
3. Wait for the timer to complete (or click "Skip" to test immediately)
4. The tomato overlay should appear:
   - Transparent background
   - Falling tomatoes animation
   - Click-through (you can click on windows behind it)
   - Press ESC to close the overlay early

## Troubleshooting

### If the overlay background is black instead of transparent:

1. Make sure you have a GPU with proper driver support
2. Try running with administrator privileges
3. Ensure Windows transparency effects are enabled in Settings > Personalization > Colors

### If build fails with "winapi" errors:

The project includes all necessary dependencies. If you still get errors, try:
```powershell
cargo clean
cargo build --release --target x86_64-pc-windows-msvc
```

### If you get "dlltool not found" error:

This happens when trying to use the GNU toolchain. Make sure to use the MSVC target:
```powershell
# Use this:
cargo build --release --target x86_64-pc-windows-msvc

# NOT this:
cargo build --release --target x86_64-pc-windows-gnu
```

## Features

The Windows build includes special handling for:
- **Focus Prevention**: The overlay never steals focus from other windows
- **Transparency**: Maintains transparency even if somehow focused
- **ESC Key**: Can close overlay with ESC even without focus
- **Click-through**: Mouse events pass through to windows below
- **No Taskbar**: Doesn't appear in taskbar or Alt+Tab

## Running as Separate Overlay Process

The application spawns the overlay as a separate process to avoid event loop conflicts:
- Main timer runs normally
- When timer completes, it launches `rust_pomodoro.exe --overlay`
- The overlay process shows the animation and exits

This ensures smooth operation without window management conflicts.