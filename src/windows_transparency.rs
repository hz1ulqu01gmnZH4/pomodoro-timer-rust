#[cfg(target_os = "windows")]
use winapi::{
    shared::{
        windef::HWND,
    },
    um::{
        winuser::{
            GetWindowLongW, SetLayeredWindowAttributes, SetWindowLongW, SetWindowPos,
            FindWindowW, GetForegroundWindow,
            GWL_EXSTYLE, LWA_ALPHA, WS_EX_LAYERED, WS_EX_TRANSPARENT,
            WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_NOACTIVATE,
            SWP_NOMOVE, SWP_NOSIZE, SWP_NOACTIVATE, SWP_SHOWWINDOW,
            HWND_TOPMOST,
        },
    },
};

/// Make a window transparent, click-through, and always on top for Windows overlay
#[cfg(target_os = "windows")]
pub fn make_overlay_transparent_and_clickthrough(hwnd: isize) {
    unsafe {
        let hwnd = hwnd as HWND;
        
        // Get current window style
        let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
        
        // Add layered, transparent (click-through), toolwindow (no taskbar), 
        // no activate (don't steal focus), and topmost
        let new_ex_style = ex_style 
            | WS_EX_LAYERED 
            | WS_EX_TRANSPARENT 
            | WS_EX_TOOLWINDOW 
            | WS_EX_NOACTIVATE 
            | WS_EX_TOPMOST;
        
        SetWindowLongW(hwnd, GWL_EXSTYLE, new_ex_style as i32);
        
        // Set full alpha (255) - transparency comes from per-pixel alpha
        SetLayeredWindowAttributes(hwnd, 0, 255, LWA_ALPHA);
        
        // Ensure the window is topmost without activating it
        SetWindowPos(
            hwnd,
            HWND_TOPMOST,
            0, 0, 0, 0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
        );
    }
}

/// Apply transparency settings for eframe/egui context
#[cfg(target_os = "windows")]
pub fn apply_overlay_transparency_for_eframe(_ctx: &egui::Context) {
    // Try to find the window by title
    unsafe {
        let window_title = "Tomato Overlay\0".encode_utf16().collect::<Vec<u16>>();
        let hwnd = FindWindowW(std::ptr::null(), window_title.as_ptr());
        
        if hwnd != std::ptr::null_mut() {
            make_overlay_transparent_and_clickthrough(hwnd as isize);
        } else {
            // Fallback: try to get the foreground window (less reliable)
            let hwnd = GetForegroundWindow();
            if hwnd != std::ptr::null_mut() {
                make_overlay_transparent_and_clickthrough(hwnd as isize);
            }
        }
    }
}

/// Dummy implementations for non-Windows platforms
#[cfg(not(target_os = "windows"))]
pub fn make_overlay_transparent_and_clickthrough(_hwnd: isize) {
    // No-op on non-Windows platforms
}

#[cfg(not(target_os = "windows"))]
pub fn apply_overlay_transparency_for_eframe(_ctx: &egui::Context) {
    // No-op on non-Windows platforms
}

/// Legacy function kept for compatibility
#[cfg(target_os = "windows")]
pub fn enable_window_transparency(hwnd: isize) {
    make_overlay_transparent_and_clickthrough(hwnd);
}

#[cfg(not(target_os = "windows"))]
pub fn enable_window_transparency(_hwnd: isize) {
    // No-op on non-Windows platforms
}

/// Set whether the window should be click-through
#[cfg(target_os = "windows")]
pub fn set_click_through(hwnd: isize, click_through: bool) {
    unsafe {
        let hwnd = hwnd as HWND;
        let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
        
        let new_ex_style = if click_through {
            (ex_style | WS_EX_TRANSPARENT) as i32
        } else {
            (ex_style & !WS_EX_TRANSPARENT) as i32
        };
        
        SetWindowLongW(hwnd, GWL_EXSTYLE, new_ex_style);
    }
}

#[cfg(not(target_os = "windows"))]
pub fn set_click_through(_hwnd: isize, _click_through: bool) {
    // No-op on non-Windows platforms
}