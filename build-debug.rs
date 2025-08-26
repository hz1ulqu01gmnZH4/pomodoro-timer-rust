fn main() {
    // Build configuration for debug mode - keeps console window visible
    #[cfg(windows)]
    {
        // Don't hide console in debug builds
        println!("cargo:rustc-link-arg-bins=/SUBSYSTEM:CONSOLE");
    }
}