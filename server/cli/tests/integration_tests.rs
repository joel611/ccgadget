/// Integration tests for CCGadget CLI
use std::process::Command;
use std::io::Write;

const BINARY_PATH: &str = "target/release/ccgadget";

/// Helper function to build binary if it doesn't exist
fn ensure_binary_exists() {
    if !std::path::Path::new(BINARY_PATH).exists() {
        println!("Building release binary for integration tests...");
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
fn test_cli_help() {
    ensure_binary_exists();
    
    let output = Command::new(BINARY_PATH)
        .args(["--help"])
        .output()
        .expect("Failed to execute binary");
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("CCGadget"));
    assert!(stdout.contains("pair"));
    assert!(stdout.contains("start"));
    assert!(stdout.contains("trigger"));
    assert!(stdout.contains("setup-hook"));
}

#[test]
fn test_pair_command_help() {
    ensure_binary_exists();
    
    let output = Command::new(BINARY_PATH)
        .args(["pair", "--help"])
        .output()
        .expect("Failed to execute binary");
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Pair with CCGadget device"));
    assert!(stdout.contains("--device"));
    assert!(stdout.contains("--force"));
}

#[test]
fn test_trigger_command_with_no_input() {
    ensure_binary_exists();
    
    let output = Command::new(BINARY_PATH)
        .args(["trigger"])
        .output()
        .expect("Failed to execute binary");
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Triggering immediate data transmission"));
    assert!(stdout.contains("Status: Payload logged for debugging"));
}

#[test]
fn test_trigger_command_with_json_input() {
    ensure_binary_exists();
    
    let test_json = r#"{"session_id": "test-session", "hook_event_name": "TestEvent"}"#;
    
    let mut child = Command::new(BINARY_PATH)
        .args(["trigger"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn command");
    
    child.stdin
        .as_mut()
        .unwrap()
        .write_all(test_json.as_bytes())
        .expect("Failed to write to stdin");
    
    let output = child.wait_with_output().expect("Failed to wait for command");
    
    // Verify command executed successfully
    assert!(output.status.success() || output.status.code() == Some(0));
    
    // Note: This test verifies the command can handle JSON input
    // The actual pairing functionality requires Bluetooth hardware
}

#[test] 
fn test_start_command_help() {
    ensure_binary_exists();
    
    let output = Command::new(BINARY_PATH)
        .args(["start", "--help"])
        .output()
        .expect("Failed to execute binary");
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Start background daemon"));
    assert!(stdout.contains("--foreground"));
    assert!(stdout.contains("--interval"));
}

#[test]
fn test_setup_hook_command_help() {
    ensure_binary_exists();
    
    let output = Command::new(BINARY_PATH)
        .args(["setup-hook", "--help"])
        .output()
        .expect("Failed to execute binary");
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("Command failed with status: {:?}, stderr: {}", output.status.code(), stderr);
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Setup Claude Code hooks"));
    assert!(stdout.contains("user"));
    assert!(stdout.contains("local"));
    assert!(stdout.contains("--scope"));
}

#[test]
fn test_log_directory_creation() {
    ensure_binary_exists();
    
    // Run trigger command to ensure log directory is created
    let _output = Command::new(BINARY_PATH)
        .args(["trigger"])
        .output()
        .expect("Failed to execute binary");
    
    // Check that log directory exists
    let home_dir = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .expect("Could not determine home directory");
    
    let log_dir = std::path::PathBuf::from(home_dir).join(".ccgadget").join("logs");
    assert!(log_dir.exists());
    assert!(log_dir.is_dir());
}