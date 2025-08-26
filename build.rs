fn main() {
    // On Windows, configure subsystem based on build type
    #[cfg(windows)]
    {
        // Check if this is a debug or release build
        let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
        
        if profile == "release" {
            // In release mode, use Windows subsystem to hide console
            println!("cargo:rustc-link-arg-bins=/SUBSYSTEM:WINDOWS");
            println!("cargo:rustc-link-arg-bins=/ENTRY:mainCRTStartup");
        }
        // In debug mode, use default console subsystem to see debug output
    }
}