// Diagnostic tool to check if transparency is supported on the current system

use std::process::Command;

pub fn check_transparency_support() -> bool {
    println!("Checking transparency support...");
    
    // Try to run wgpu-info if available
    if let Ok(output) = Command::new("wgpu-info")
        .arg("--raw")
        .output() 
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("wgpu-info output:\n{}", stdout);
        
        // Check if PreMultiplied or PostMultiplied alpha modes are supported
        let has_alpha = stdout.contains("PreMultiplied") || stdout.contains("PostMultiplied");
        
        if !has_alpha {
            println!("WARNING: Your GPU driver does not support transparent windows!");
            println!("This is a known issue with AMD drivers 23.10.x and newer.");
            println!("Workarounds:");
            println!("1. Use OpenGL renderer (already implemented as fallback)");
            println!("2. Downgrade AMD driver to 23.8.2 or earlier");
            println!("3. Try disabling MPO with: reg add HKLM\\SOFTWARE\\Microsoft\\Windows\\Dwm /v OverlayTestMode /t REG_DWORD /d 5");
            return false;
        }
    } else {
        println!("wgpu-info not found. Install with: cargo install wgpu-info");
    }
    
    true
}

pub fn get_gpu_info() {
    // Try to get GPU info from Windows
    if let Ok(output) = Command::new("wmic")
        .args(&["path", "win32_VideoController", "get", "name,driverversion"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("GPU Info:\n{}", stdout);
    }
}