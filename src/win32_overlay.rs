#[cfg(windows)]
use winapi::{
    shared::{
        minwindef::{HINSTANCE, UINT, WPARAM, LPARAM, LRESULT},
        windef::{HWND, HBRUSH, RECT},
    },
    um::{
        winuser::{
            RegisterClassW, CreateWindowExW, ShowWindow, UpdateWindow,
            DefWindowProcW, PostQuitMessage, GetMessageW, TranslateMessage,
            DispatchMessageW, LoadCursorW, GetSystemMetrics, SetLayeredWindowAttributes,
            SetTimer, KillTimer, InvalidateRect, BeginPaint, EndPaint,
            WNDCLASSW, MSG, WS_EX_LAYERED, WS_EX_TRANSPARENT, WS_EX_TOPMOST,
            WS_POPUP, SW_SHOW, SM_CXSCREEN, SM_CYSCREEN, LWA_ALPHA, LWA_COLORKEY,
            WM_CREATE, WM_DESTROY, WM_TIMER, WM_PAINT, WM_KEYDOWN, VK_ESCAPE,
            CS_HREDRAW, CS_VREDRAW, IDC_ARROW, PAINTSTRUCT,
        },
        wingdi::{CreateSolidBrush, RGB},
        libloaderapi::GetModuleHandleW,
    },
};
use std::ptr::null_mut;
use std::mem::zeroed;

#[cfg(windows)]
pub fn show_win32_overlay() {
    unsafe {
        let h_instance = GetModuleHandleW(null_mut());
        let class_name = wide_string("TomatoRainOverlay");
        
        // Register window class
        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(window_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: h_instance,
            hIcon: null_mut(),
            hCursor: LoadCursorW(null_mut(), IDC_ARROW),
            hbrBackground: null_mut(),
            lpszMenuName: null_mut(),
            lpszClassName: class_name.as_ptr(),
        };
        
        RegisterClassW(&wc);
        
        // Create layered window
        let hwnd = CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST,
            class_name.as_ptr(),
            wide_string("Tomato Rain").as_ptr(),
            WS_POPUP,
            0, 0,
            GetSystemMetrics(SM_CXSCREEN),
            GetSystemMetrics(SM_CYSCREEN),
            null_mut(),
            null_mut(),
            h_instance,
            null_mut(),
        );
        
        // Set transparency
        SetLayeredWindowAttributes(
            hwnd,
            RGB(0, 0, 0), // Black as transparent color
            200,           // Alpha value (0-255)
            LWA_COLORKEY | LWA_ALPHA,
        );
        
        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);
        
        // Start animation timer
        SetTimer(hwnd, 1, 16, None); // ~60 FPS
        
        // Message loop
        let mut msg: MSG = zeroed();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

#[cfg(windows)]
unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            0
        }
        WM_TIMER => {
            InvalidateRect(hwnd, null_mut(), 0);
            0
        }
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = zeroed();
            let hdc = BeginPaint(hwnd, &mut ps);
            
            // Here you would draw tomatoes using GDI+
            // For now, this is just a transparent overlay
            
            EndPaint(hwnd, &ps);
            0
        }
        WM_KEYDOWN => {
            if wparam == VK_ESCAPE as usize {
                PostQuitMessage(0);
            }
            0
        }
        WM_DESTROY => {
            KillTimer(hwnd, 1);
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

#[cfg(windows)]
fn wide_string(s: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    use std::ffi::OsStr;
    
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

#[cfg(not(windows))]
pub fn show_win32_overlay() {
    println!("Win32 overlay is only available on Windows");
}