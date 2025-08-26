use std::panic;

fn main() {
    // Set up panic handler to show errors on Windows
    panic::set_hook(Box::new(|panic_info| {
        let msg = format!("Application panicked:\n{}", panic_info);
        eprintln!("{}", msg);
        
        #[cfg(windows)]
        {
            use std::ffi::OsStr;
            use std::os::windows::ffi::OsStrExt;
            use std::ptr;
            
            let wide: Vec<u16> = OsStr::new(&msg).encode_wide().chain(std::iter::once(0)).collect();
            unsafe {
                winapi::um::winuser::MessageBoxW(
                    ptr::null_mut(),
                    wide.as_ptr(),
                    [82, 117, 115, 116, 32, 80, 111, 109, 111, 100, 111, 114, 111, 32, 69, 114, 114, 111, 114, 0].as_ptr(), // "Rust Pomodoro Error"
                    winapi::um::winuser::MB_OK | winapi::um::winuser::MB_ICONERROR,
                );
            }
        }
    }));

    // Run the actual main
    if let Err(e) = rust_pomodoro::run() {
        eprintln!("Error: {}", e);
        #[cfg(windows)]
        {
            let msg = format!("Failed to start application:\n{}", e);
            use std::ffi::OsStr;
            use std::os::windows::ffi::OsStrExt;
            use std::ptr;
            
            let wide: Vec<u16> = OsStr::new(&msg).encode_wide().chain(std::iter::once(0)).collect();
            unsafe {
                winapi::um::winuser::MessageBoxW(
                    ptr::null_mut(),
                    wide.as_ptr(),
                    [82, 117, 115, 116, 32, 80, 111, 109, 111, 100, 111, 114, 111, 32, 69, 114, 114, 111, 114, 0].as_ptr(),
                    winapi::um::winuser::MB_OK | winapi::um::winuser::MB_ICONERROR,
                );
            }
        }
        std::process::exit(1);
    }
}