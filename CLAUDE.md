# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

### Electron Application (JavaScript)
- **Run the app**: `npm start`
- **Run on WSL**: `npm start:wsl` (uses GPU-disabled flags for WSL compatibility)
- **Build for distribution**: `npm run dist`
- **Package directory**: `npm run pack`

### Rust Application
Located in `rust-pomodoro/` directory:
- **Debug build**: `cargo build`
- **Release build**: `cargo build --release`
- **Run directly**: `cargo run --release`
- **Build for Windows**: Use `build-windows.bat` or `build-windows-gnu.sh`

## Architecture Overview

This repository contains two implementations of a Pomodoro timer:

### 1. Electron Implementation (Main)
The primary implementation using Electron with the following architecture:

- **main.js**: Main process handling window creation, system tray, and IPC communication
  - Creates main window (400x600, non-resizable)
  - Manages transparent overlay window for tomato animation
  - Handles system tray with context menu
  - Manages notifications (cross-platform audio and system notifications)

- **timer.js**: Core timer logic in renderer process
  - PomodoroTimer class managing state and UI updates
  - Session types: work (25min), short break (5min), long break (15min after 4 sessions)
  - Settings persistence via localStorage
  - Progress visualization with SVG circle

- **overlay.js/overlay.html**: Falling tomato animation
  - Uses p5.js for canvas-based animation
  - Creates full-screen transparent overlay
  - Auto-closes after 10 seconds

- **preload.js**: Bridge between main and renderer processes
  - Exposes safe IPC methods via window.electronAPI
  - Handles timer control commands and notifications

### 2. Rust Implementation (Alternative)
Native implementation in `rust-pomodoro/` using:
- **egui/eframe**: Immediate mode GUI framework
- **wgpu**: Hardware-accelerated graphics
- **notify-rust**: Cross-platform notifications
- Multiple overlay strategies for different platforms (Bevy, Win32 API, native window)

## Key Interaction Patterns

### IPC Communication
- Renderer → Main: `window.electronAPI.timerComplete(sessionType)`
- Main → Renderer: `timer-control` events ('start', 'pause', 'reset')
- Tray tooltip updates: `window.electronAPI.updateTrayTooltip(text)`

### State Management
- Timer state maintained in `PomodoroTimer` class instance
- Settings saved to localStorage on change
- Session progression: work → short break (×3) → work → long break → repeat

### Platform-Specific Handling
- Audio playback varies by platform (PowerShell/afplay/aplay)
- WSL requires special flags: `--disable-gpu --disable-software-rasterizer`
- Rust version includes platform-specific overlay implementations

## Important Implementation Details

- The Electron app hides to tray on close (doesn't quit)
- Overlay window is click-through and always-on-top
- Timer continues running when window is hidden
- Settings changes only apply when timer is not running
- Session count determines break type (every 4th is long break)