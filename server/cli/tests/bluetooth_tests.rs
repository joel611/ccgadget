/// Bluetooth pairing integration tests for CCGadget CLI
use std::process::Command;
use std::time::Duration;
use std::thread;

const BINARY_PATH: &str = "target/release/ccgadget";

/// Helper function to build binary if it doesn't exist
fn ensure_binary_exists() {
    if !std::path::Path::new(BINARY_PATH).exists() {
        println!("Building release binary for Bluetooth tests...");
        let output = Command::new("cargo")
            .args(["build", "--release"])
            .output()
            .expect("Failed to build release binary");
        
        if !output.status.success() {
            panic!("Failed to build release binary: {}", String::from_utf8_lossy(&output.stderr));
        }
    }
}

#[test]
fn test_pair_command_basic_execution() {
    ensure_binary_exists();
    
    // Test pair command with a non-existent device (should fail gracefully)
    let output = Command::new(BINARY_PATH)
        .args(["pair", "--device", "NonExistentDevice"])
        .output()
        .expect("Failed to execute pair command");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Should start pairing process
    assert!(stdout.contains("Pairing with CCGadget device"));
    assert!(stdout.contains("Target device: NonExistentDevice"));
    
    // Should eventually fail to find the device (expected behavior)
    println!("Pair command output: {}", stdout);
    if !stderr.is_empty() {
        println!("Pair command stderr: {}", stderr);
    }
}

#[test]
fn test_pair_command_force_flag() {
    ensure_binary_exists();
    
    let output = Command::new(BINARY_PATH)
        .args(["pair", "--device", "TestDevice", "--force"])
        .output()
        .expect("Failed to execute pair command");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Should show force pairing enabled
    assert!(stdout.contains("Pairing with CCGadget device"));
    assert!(stdout.contains("Force pairing enabled"));
    assert!(stdout.contains("Target device: TestDevice"));
}

#[test]
fn test_pair_command_scanning_mode() {
    ensure_binary_exists();
    
    // Test pair command in demo mode to avoid Bluetooth hanging
    let output = Command::new(BINARY_PATH)
        .args(["pair"])
        .env("CCGADGET_DEMO_MODE", "1")
        .output()
        .expect("Failed to execute pair command");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Should start scanning process in demo mode
    assert!(stdout.contains("Pairing with CCGadget device"));
    assert!(stdout.contains("demo/test mode"));
    assert!(stdout.contains("Found 3 CCGadget device(s)"));
    println!("Demo scanning mode output: {}", stdout);
}

#[test]
fn test_bluetooth_error_handling() {
    ensure_binary_exists();
    
    // Test with invalid device address format
    let output = Command::new(BINARY_PATH)
        .args(["pair", "--device", "invalid::address::format"])
        .output()
        .expect("Failed to execute pair command");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Should handle invalid device gracefully
    assert!(stdout.contains("Pairing with CCGadget device"));
    println!("Error handling output: {}", stdout);
    if !stderr.is_empty() {
        println!("Error handling stderr: {}", stderr);
    }
}

#[ignore] // Marked as ignore since this requires actual Bluetooth hardware
#[test]
fn test_real_bluetooth_scan() {
    ensure_binary_exists();
    
    println!("⚠️ This test requires actual Bluetooth hardware and may take time...");
    
    let _output = Command::new(BINARY_PATH)
        .args(["pair"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to start pair command");
    
    // In a real test, you could send "0" to cancel the pairing
    // For now, we just verify the process can start
    thread::sleep(Duration::from_secs(2));
    
    println!("✅ Bluetooth scanning process started successfully");
}


#[test]
fn test_pair_help_contains_bluetooth_info() {
    ensure_binary_exists();
    
    let output = Command::new(BINARY_PATH)
        .args(["pair", "--help"])
        .output()
        .expect("Failed to execute pair help");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Should mention Bluetooth functionality
    assert!(stdout.contains("Bluetooth"));
    assert!(stdout.contains("device"));
    assert!(stdout.contains("pair"));
}