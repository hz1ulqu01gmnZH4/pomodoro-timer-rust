#[cfg(target_os = "windows")]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(target_os = "windows")]
use std::sync::OnceLock;

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{
    CallWindowProcW, DefWindowProcW,
    GetForegroundWindow, GetWindowLongPtrW,
    SetForegroundWindow, SetLayeredWindowAttributes, SetWindowLongPtrW, SetWindowPos,
    ShowWindow, GWL_EXSTYLE, GWLP_WNDPROC, HWND_TOPMOST, LWA_ALPHA, MA_NOACTIVATE,
    SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW, SW_SHOWNOACTIVATE,
    WM_ACTIVATE, WM_ERASEBKGND, WM_MOUSEACTIVATE, WM_NCACTIVATE, WM_SETFOCUS,
    WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT,
};
#[cfg(target_os = "windows")]
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_ESCAPE};

#[cfg(target_os = "windows")]
static ORIG_WNDPROC: OnceLock<isize> = OnceLock::new();
#[cfg(target_os = "windows")]
static PREV_FG: OnceLock<HWND> = OnceLock::new();
#[cfg(target_os = "windows")]
static CLOSE_REQUESTED: AtomicBool = AtomicBool::new(false);

/// Install overlay window styles and behavior to prevent focus and maintain transparency
#[cfg(target_os = "windows")]
pub unsafe fn install_overlay_window(hwnd: HWND, preferred_return_focus: Option<HWND>) {
    // Remember whatever had focus before the overlay shows
    if PREV_FG.get().is_none() {
        let prev = preferred_return_focus.unwrap_or(GetForegroundWindow());
        let _ = PREV_FG.set(prev);
    }

    // Add styles: layered + transparent + toolwindow + noactivate + topmost
    let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
    let mut new_ex_style = ex_style;
    new_ex_style |= WS_EX_LAYERED.0 
        | WS_EX_TRANSPARENT.0 
        | WS_EX_TOOLWINDOW.0 
        | WS_EX_NOACTIVATE.0 
        | WS_EX_TOPMOST.0;
    SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_ex_style as isize);

    // Per-pixel alpha compositing on (keep 255 so your framebuffer alpha decides)
    SetLayeredWindowAttributes(hwnd, windows::Win32::Foundation::COLORREF(0), 255, LWA_ALPHA);

    // Topmost but do not activate
    SetWindowPos(
        hwnd,
        HWND_TOPMOST,
        0,
        0,
        0,
        0,
        SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
    );
    
    // Show without activation
    ShowWindow(hwnd, SW_SHOWNOACTIVATE);

    // Subclass to kill activation and avoid background erase
    if ORIG_WNDPROC.get().is_none() {
        let orig = GetWindowLongPtrW(hwnd, GWLP_WNDPROC);
        let _ = ORIG_WNDPROC.set(orig);
        SetWindowLongPtrW(hwnd, GWLP_WNDPROC, overlay_wndproc as isize);
    }
}

/// Restore original window procedure when closing
#[cfg(target_os = "windows")]
pub unsafe fn uninstall_overlay_window(hwnd: HWND) {
    if let Some(orig) = ORIG_WNDPROC.get() {
        SetWindowLongPtrW(hwnd, GWLP_WNDPROC, *orig);
    }
}

/// Check if close was requested (for ESC handling)
#[cfg(target_os = "windows")]
pub fn take_close_requested() -> bool {
    CLOSE_REQUESTED.swap(false, Ordering::SeqCst)
}

/// Check if ESC key is currently pressed (globally, no focus required)
#[cfg(target_os = "windows")]
pub fn esc_is_down() -> bool {
    unsafe {
        // GetAsyncKeyState high bit indicates key is down globally (no focus required)
        // Use i16 cast to avoid overflow warning
        (GetAsyncKeyState(VK_ESCAPE.0 as i32) as u16 & 0x8000) != 0
    }
}

/// Try to return focus to the remembered window immediately
#[cfg(target_os = "windows")]
unsafe fn return_focus() {
    if let Some(prev) = PREV_FG.get().copied() {
        if prev.0 != 0 {
            // Simply try to set foreground window
            // Note: AttachThreadInput and SetFocus are not available in windows crate v0.52
            // but SetForegroundWindow should be sufficient for most cases
            let _ = SetForegroundWindow(prev);
        }
    }
}

/// Custom window procedure to prevent activation and background erasing
#[cfg(target_os = "windows")]
extern "system" fn overlay_wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match msg {
            // Prevent activation on mouse interactions
            WM_MOUSEACTIVATE => {
                return LRESULT(MA_NOACTIVATE as isize);
            }
            // Don't let Windows change "active" painting of non-client area
            WM_NCACTIVATE => {
                // Tell the system we handled it; do not change active state visuals
                return LRESULT(1);
            }
            // If anything tried to activate us, immediately bounce focus back
            WM_ACTIVATE => {
                // LOWORD(wparam) != WA_INACTIVE means activation attempt
                if (wparam.0 & 0xFFFF) != 0 {
                    return_focus();
                }
                return LRESULT(0);
            }
            // Also bounce focus if we somehow get it
            WM_SETFOCUS => {
                return_focus();
                return LRESULT(0);
            }
            // Avoid background erase that can produce black/opaque flashes
            WM_ERASEBKGND => {
                return LRESULT(1);
            }
            _ => {}
        }

        // Default: forward to original WndProc
        if let Some(orig) = ORIG_WNDPROC.get().copied() {
            let orig_proc = std::mem::transmute::<isize, unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT>(orig);
            return CallWindowProcW(Some(orig_proc), hwnd, msg, wparam, lparam);
        }
        DefWindowProcW(hwnd, msg, wparam, lparam)
    }
}

// Non-Windows stubs
#[cfg(not(target_os = "windows"))]
pub unsafe fn install_overlay_window(_hwnd: isize, _preferred_return_focus: Option<isize>) {}

#[cfg(not(target_os = "windows"))]
pub unsafe fn uninstall_overlay_window(_hwnd: isize) {}

#[cfg(not(target_os = "windows"))]
pub fn take_close_requested() -> bool { false }

#[cfg(not(target_os = "windows"))]
pub fn esc_is_down() -> bool { false }