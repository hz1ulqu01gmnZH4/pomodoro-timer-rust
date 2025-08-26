#!/bin/bash

# Test script to manually trigger the tomato overlay

cd "$(dirname "$0")"

# Build the test program
cat > src/bin/test_overlay.rs << 'EOF'
use std::thread;
use std::time::Duration;

mod transparent_overlay {
    include!("../transparent_overlay.rs");
}

fn main() {
    println!("Testing tomato overlay...");
    println!("Starting overlay in 2 seconds...");
    
    thread::sleep(Duration::from_secs(2));
    
    println!("Launching overlay window...");
    transparent_overlay::TransparentOverlay::show();
    
    println!("Overlay test complete!");
}
EOF

# Compile the test
echo "Building test overlay..."
cargo build --bin test_overlay --target x86_64-unknown-linux-gnu 2>&1

# Run the test
echo "Running test overlay..."
DISPLAY=:0 ./target/x86_64-unknown-linux-gnu/debug/test_overlay