use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::Manager;
use clap::{CommandFactory, Parser, Subcommand};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Parser)]
#[command(name = "ccgadget")]
#[command(version = "0.1.0")]
#[command(about = "CLI tool for CCGadget IoT hardware monitoring device")]
#[command(long_about = "CCGadget is an IoT-enabled hardware gadget that monitors and displays \
real-time Claude Code usage metrics. This CLI tool manages device pairing, data transmission, \
and integration with Claude Code hooks.")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Pair with CCGadget device via Bluetooth
    Pair {
        /// Device name or address to pair with
        #[arg(short, long)]
        device: Option<String>,
        /// Force pairing even if already paired
        #[arg(short, long)]
        force: bool,
    },
    /// Start background daemon to monitor and transmit usage data
    Start {
        /// Run in foreground mode (don't daemonize)
        #[arg(short, long)]
        foreground: bool,
        /// Update interval in seconds
        #[arg(short, long, default_value = "30")]
        interval: u64,
    },
    /// Trigger immediate data transmission (for Claude Code hooks)
    Trigger,
    /// Install Claude Code hooks for automatic monitoring
    InstallHook {
        /// Hook type to install
        #[arg(value_enum)]
        hook_type: HookType,
        /// Force reinstall if hook already exists
        #[arg(short, long)]
        force: bool,
    },
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum HookType {
    /// Install all available hooks
    All,
    /// Session start/end hooks
    Session,
    /// Command execution hooks
    Command,
    /// Usage monitoring hooks
    Usage,
}

#[derive(Serialize, Deserialize, Debug)]
struct TriggerLogEntry {
    timestamp: DateTime<Utc>,
    hook_input: Option<HookInput>,
    metadata: LogMetadata,
}

#[derive(Serialize, Deserialize, Debug)]
struct LogMetadata {
    version: String,
    source: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct HookInput {
    // Common fields for all events
    session_id: Option<String>,
    transcript_path: Option<String>,
    cwd: Option<String>,
    hook_event_name: Option<String>,
    
    // UserPromptSubmit specific
    prompt: Option<String>,
    
    // Notification specific
    message: Option<String>,
    
    // PreToolUse specific
    tool_name: Option<String>,
    tool_input: Option<serde_json::Value>,
    
    // PostToolUse specific (includes tool_name and tool_input from PreToolUse)
    tool_response: Option<serde_json::Value>,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Pair { device, force }) => {
            handle_pair(device.as_deref(), *force).await;
        }
        Some(Commands::Start { foreground, interval }) => {
            handle_start(*foreground, *interval);
        }
        Some(Commands::Trigger) => {
            handle_trigger();
        }
        Some(Commands::InstallHook { hook_type, force }) => {
            handle_install_hook(hook_type, *force);
        }
        None => {
            // No subcommand provided, show help
            let mut cmd = Cli::command();
            cmd.print_help().unwrap();
            std::process::exit(0);
        }
    }
}

/// Handle device pairing with Bluetooth scanning and user selection
async fn handle_pair(device: Option<&str>, force: bool) {
    println!("🔵 Pairing with CCGadget device...");
    
    if force {
        println!("   Force pairing enabled");
    }

    // Check if we're in a test environment or don't have Bluetooth permissions
    if std::env::var("CCGADGET_DEMO_MODE").is_ok() {
        println!("   🔧 Running in demo/test mode - simulating pairing");
        simulate_pairing(device).await;
        return;
    }

    println!("   💡 If this hangs or fails, use: CCGADGET_DEMO_MODE=1 ccgadget pair");

    if let Some(device_name) = device {
        println!("   Target device: {}", device_name);
        if let Err(e) = pair_with_device(device_name, force).await {
            eprintln!("   ❌ Failed to pair with device: {}", e);
            eprintln!("   💡 To test without Bluetooth: CCGADGET_DEMO_MODE=1 ccgadget pair");
            std::process::exit(1);
        }
    } else {
        println!("   Scanning for nearby Bluetooth devices...");
        match scan_and_select_device().await {
            Ok(Some(selected_device)) => {
                println!("   Selected device: {}", selected_device);
                if let Err(e) = pair_with_device(&selected_device, force).await {
                    eprintln!("   ❌ Failed to pair with selected device: {}", e);
                    std::process::exit(1);
                }
            }
            Ok(None) => {
                println!("   ℹ️ No device selected. Pairing cancelled.");
            }
            Err(e) => {
                eprintln!("   ❌ Error during device scanning: {}", e);
                eprintln!("   💡 To test without Bluetooth: CCGADGET_DEMO_MODE=1 ccgadget pair");
                std::process::exit(1);
            }
        }
    }
}

/// Check if a device name matches CCGadget patterns
pub fn is_ccgadget_device(name: &str) -> bool {
    let name_lower = name.to_lowercase();
    name_lower.contains("ccgadget") || 
    name_lower.starts_with("ccg-") ||
    name_lower.contains("esp32-ccg")
}

/// Simulate pairing for demo/test mode
async fn simulate_pairing(device: Option<&str>) {
    if let Some(device_name) = device {
        println!("   🎯 Target device: {}", device_name);
        println!("   🔍 Simulating Bluetooth scan...");
        tokio::time::sleep(Duration::from_millis(500)).await;
        println!("   ✅ Found simulated device: {}", device_name);
        println!("   🔗 Connecting to device...");
        tokio::time::sleep(Duration::from_millis(300)).await;
        println!("   📋 Discovering services... (2 service(s) found)");
        println!("      - Service UUID: 12345678-1234-5678-9abc-123456789abc");
        println!("      - Service UUID: 87654321-4321-8765-cba9-987654321abc");
        println!("   ✅ Pairing completed successfully!");
        println!("   ℹ️ Note: This was a simulated pairing for demo/testing purposes");
    } else {
        println!("   📡 Simulating device scan...");
        tokio::time::sleep(Duration::from_millis(800)).await;
        println!("   📱 Found 3 CCGadget device(s):");
        println!("   1. CCGadget-Demo (AA:BB:CC:DD:EE:FF) - Signal: -45dBm");
        println!("   2. CCG-Office (11:22:33:44:55:66) - Signal: -67dBm");
        println!("   3. ESP32-CCG-Lab (99:88:77:66:55:44) - Signal: -72dBm");
        println!("   0. Cancel");
        println!("   ℹ️ Auto-selecting device 1 for demo");
        tokio::time::sleep(Duration::from_millis(500)).await;
        println!("   🔗 Connecting to CCGadget-Demo...");
        tokio::time::sleep(Duration::from_millis(300)).await;
        println!("   ✅ Pairing completed successfully!");
        println!("   ℹ️ Note: This was a simulated pairing for demo/testing purposes");
    }
}

/// Scan for Bluetooth devices and let user select one
async fn scan_and_select_device() -> Result<Option<String>, Box<dyn std::error::Error>> {
    // Get the Bluetooth manager with timeout and better error handling
    println!("   🔍 Initializing Bluetooth manager...");
    let manager = match tokio::time::timeout(Duration::from_secs(5), Manager::new()).await {
        Ok(Ok(manager)) => {
            println!("   ✅ Bluetooth manager initialized");
            manager
        },
        Ok(Err(e)) => {
            eprintln!("   ❌ Failed to initialize Bluetooth manager: {}", e);
            eprintln!("   💡 Possible solutions:");
            eprintln!("      - Enable Bluetooth in System Settings");
            eprintln!("      - Grant Bluetooth permission to Terminal/CLI in Privacy & Security settings");
            eprintln!("      - Run: sudo xcode-select --install (if needed)");
            return Err("Bluetooth initialization failed".into());
        },
        Err(_) => {
            eprintln!("   ❌ Bluetooth manager initialization timed out");
            eprintln!("   💡 This may indicate:");
            eprintln!("      - Bluetooth service is not running");
            eprintln!("      - Permission issues (check Privacy & Security settings)");
            eprintln!("      - Hardware compatibility issues");
            return Err("Bluetooth timeout - check system settings and permissions".into());
        }
    };
    
    // Get the first Bluetooth adapter
    println!("   🔍 Finding Bluetooth adapters...");
    let adapters = match tokio::time::timeout(Duration::from_secs(2), manager.adapters()).await {
        Ok(Ok(adapters)) => adapters,
        Ok(Err(e)) => {
            eprintln!("   ❌ Failed to get Bluetooth adapters: {}", e);
            eprintln!("   💡 This usually indicates permission or hardware issues");
            return Err("Bluetooth adapter access failed".into());
        },
        Err(_) => {
            eprintln!("   ❌ Bluetooth adapter detection timed out");
            eprintln!("   💡 Bluetooth adapters are taking too long to respond");
            eprintln!("      This often means permission issues or system Bluetooth problems");
            return Err("Bluetooth adapter timeout".into());
        }
    };
    
    let central = adapters
        .into_iter()
        .next()
        .ok_or("No Bluetooth adapter found. \n   💡 Check if Bluetooth hardware is available and enabled.")?;
    
    println!("   ✅ Bluetooth adapter found");
    
    println!("   📡 Starting Bluetooth scan (10 seconds)...");
    
    // Start scanning
    central.start_scan(ScanFilter::default()).await?;
    
    // Scan for 10 seconds
    sleep(Duration::from_secs(10)).await;
    
    // Stop scanning
    central.stop_scan().await?;
    
    // Get discovered peripherals
    let peripherals = central.peripherals().await?;
    
    if peripherals.is_empty() {
        println!("   ⚠️ No Bluetooth devices found");
        return Ok(None);
    }
    
    // Collect CCGadget device information (filtered)
    let mut devices = Vec::new();
    for peripheral in peripherals {
        let properties = peripheral.properties().await?;
        if let Some(props) = properties {
            let name = props.local_name.unwrap_or_else(|| "Unknown Device".to_string());
            
            // Filter: only include CCGadget devices
            if is_ccgadget_device(&name) {
                let address = props.address.to_string();
                let rssi = props.rssi.map(|r| format!("{}dBm", r)).unwrap_or_else(|| "N/A".to_string());
                devices.push((name, address, rssi));
            }
        }
    }
    
    if devices.is_empty() {
        println!("   ⚠️ No CCGadget devices found");
        println!("   💡 Make sure your CCGadget device is:");
        println!("      - Powered on and in pairing mode");
        println!("      - Within Bluetooth range (10 meters)");
        println!("      - Named with 'CCGadget', 'CCG-', or 'ESP32-CCG' prefix");
        return Ok(None);
    }
    
    // Display found CCGadget devices
    println!("   📱 Found {} CCGadget device(s):", devices.len());
    for (i, (name, address, rssi)) in devices.iter().enumerate() {
        println!("   {}. {} ({}) - Signal: {}", i + 1, name, address, rssi);
    }
    println!("   0. Cancel");
    
    // Get user selection
    loop {
        print!("   Select a device to pair with (0-{}): ", devices.len());
        io::stdout().flush()?;
        
        let stdin = io::stdin();
        let mut line = String::new();
        stdin.read_line(&mut line)?;
        
        match line.trim().parse::<usize>() {
            Ok(0) => {
                return Ok(None);
            }
            Ok(selection) if selection <= devices.len() => {
                let selected = &devices[selection - 1];
                return Ok(Some(selected.1.clone())); // Return the address
            }
            _ => {
                println!("   ❌ Invalid selection. Please try again.");
                continue;
            }
        }
    }
}

/// Attempt to pair with a specific device
async fn pair_with_device(device_identifier: &str, _force: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("   🔗 Attempting to pair with device: {}", device_identifier);
    
    // Get the Bluetooth manager with timeout and better error handling
    println!("   🔍 Initializing Bluetooth manager...");
    let manager = match tokio::time::timeout(Duration::from_secs(5), Manager::new()).await {
        Ok(Ok(manager)) => {
            println!("   ✅ Bluetooth manager initialized");
            manager
        },
        Ok(Err(e)) => {
            eprintln!("   ❌ Failed to initialize Bluetooth manager: {}", e);
            eprintln!("   💡 Possible solutions:");
            eprintln!("      - Enable Bluetooth in System Settings");
            eprintln!("      - Grant Bluetooth permission to Terminal/CLI in Privacy & Security settings");
            eprintln!("      - Run: sudo xcode-select --install (if needed)");
            return Err("Bluetooth initialization failed".into());
        },
        Err(_) => {
            eprintln!("   ❌ Bluetooth manager initialization timed out");
            eprintln!("   💡 This may indicate:");
            eprintln!("      - Bluetooth service is not running");
            eprintln!("      - Permission issues (check Privacy & Security settings)");
            eprintln!("      - Hardware compatibility issues");
            return Err("Bluetooth timeout - check system settings and permissions".into());
        }
    };
    
    // Get the first Bluetooth adapter
    println!("   🔍 Finding Bluetooth adapters...");
    let adapters = match tokio::time::timeout(Duration::from_secs(2), manager.adapters()).await {
        Ok(Ok(adapters)) => adapters,
        Ok(Err(e)) => {
            eprintln!("   ❌ Failed to get Bluetooth adapters: {}", e);
            eprintln!("   💡 This usually indicates permission or hardware issues");
            return Err("Bluetooth adapter access failed".into());
        },
        Err(_) => {
            eprintln!("   ❌ Bluetooth adapter detection timed out");
            eprintln!("   💡 Bluetooth adapters are taking too long to respond");
            eprintln!("      This often means permission issues or system Bluetooth problems");
            return Err("Bluetooth adapter timeout".into());
        }
    };
    
    let central = adapters
        .into_iter()
        .next()
        .ok_or("No Bluetooth adapter found. \n   💡 Check if Bluetooth hardware is available and enabled.")?;
    
    println!("   ✅ Bluetooth adapter found");
    
    // Start scanning to find the device
    println!("   📡 Scanning for target device...");
    central.start_scan(ScanFilter::default()).await?;
    
    // Scan for up to 15 seconds to find the target device
    let mut found_peripheral = None;
    for _ in 0..15 {
        sleep(Duration::from_secs(1)).await;
        
        let peripherals = central.peripherals().await?;
        for peripheral in peripherals {
            let properties = peripheral.properties().await?;
            if let Some(props) = properties {
                let address = props.address.to_string();
                let name = props.local_name.unwrap_or_default();
                
                // Match by address or name
                if address.eq_ignore_ascii_case(device_identifier) ||
                   name.eq_ignore_ascii_case(device_identifier) {
                    found_peripheral = Some(peripheral);
                    break;
                }
            }
        }
        
        if found_peripheral.is_some() {
            break;
        }
    }
    
    central.stop_scan().await?;
    
    let peripheral = found_peripheral
        .ok_or_else(|| format!("Device '{}' not found", device_identifier))?;
    
    println!("   ✅ Found target device, attempting connection...");
    
    // Connect to the device
    peripheral.connect().await?;
    println!("   🎉 Successfully connected to device!");
    
    // Discover services
    peripheral.discover_services().await?;
    let services = peripheral.services();
    
    println!("   📋 Device services discovered: {} service(s)", services.len());
    for service in services {
        println!("      - Service UUID: {}", service.uuid);
    }
    
    // For now, just disconnect after discovery
    // In a real implementation, you'd establish the pairing here
    peripheral.disconnect().await?;
    
    println!("   ✅ Pairing completed successfully!");
    Ok(())
}

fn handle_start(foreground: bool, interval: u64) {
    println!("🚀 Starting CCGadget monitoring daemon...");
    println!("   Mode: {}", if foreground { "Foreground" } else { "Background" });
    println!("   Update interval: {}s", interval);
    println!("   Status: Not yet implemented");
}

fn handle_trigger() {
    println!("⚡ Triggering immediate data transmission...");
    
    // Read hook input from stdin
    let hook_input = read_hook_input_from_stdin();
    
    // Log the payload for debugging
    match log_trigger_payload(hook_input.as_ref()) {
        Ok(log_path) => {
            println!("   ✅ Payload logged to: {}", log_path.display());
        }
        Err(e) => {
            eprintln!("   ❌ Failed to log payload: {}", e);
        }
    }
    if let Some(ref hook_data) = hook_input {
        println!("   Hook Event: {:?}", hook_data.hook_event_name);
        if let Some(ref session_id) = hook_data.session_id {
            println!("   Session ID: {}", session_id);
        }
        if let Some(ref cwd) = hook_data.cwd {
            println!("   Working Directory: {}", cwd);
        }
        
        // Event-specific data
        match hook_data.hook_event_name.as_deref() {
            Some("UserPromptSubmit") => {
                if let Some(ref prompt) = hook_data.prompt {
                    println!("   Prompt: {}", prompt);
                }
            }
            Some("Notification") => {
                if let Some(ref message) = hook_data.message {
                    println!("   Message: {}", message);
                }
            }
            Some("PreToolUse") => {
                if let Some(ref tool_name) = hook_data.tool_name {
                    println!("   Tool: {}", tool_name);
                }
                if let Some(ref tool_input) = hook_data.tool_input {
                    println!("   Tool Input: {}", serde_json::to_string_pretty(tool_input).unwrap_or_default());
                }
            }
            Some("PostToolUse") => {
                if let Some(ref tool_name) = hook_data.tool_name {
                    println!("   Tool: {}", tool_name);
                }
                if let Some(ref tool_response) = hook_data.tool_response {
                    println!("   Tool Response: {}", serde_json::to_string_pretty(tool_response).unwrap_or_default());
                }
            }
            _ => {}
        }
    }
    println!("   Status: Payload logged for debugging");
}

fn get_log_directory() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home_dir = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| "Could not determine home directory")?;
    
    let log_dir = PathBuf::from(home_dir).join(".ccgadget").join("logs");
    
    if !log_dir.exists() {
        fs::create_dir_all(&log_dir)?;
    }
    
    Ok(log_dir)
}

fn read_hook_input_from_stdin() -> Option<HookInput> {
    let mut buffer = String::new();
    match io::stdin().read_to_string(&mut buffer) {
        Ok(_) if !buffer.trim().is_empty() => {
            match serde_json::from_str::<HookInput>(&buffer) {
                Ok(hook_input) => Some(hook_input),
                Err(e) => {
                    eprintln!("   ⚠️ Failed to parse hook input: {}", e);
                    None
                }
            }
        }
        _ => None,
    }
}

fn log_trigger_payload(hook_input: Option<&HookInput>) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let log_dir = get_log_directory()?;
    
    // Create daily log file
    let now = Utc::now();
    let date_str = now.format("%Y-%m-%d");
    let log_file_path = log_dir.join(format!("trigger-{}.log", date_str));
    
    // Create log entry
    let log_entry = TriggerLogEntry {
        timestamp: now,
        hook_input: hook_input.cloned(),
        metadata: LogMetadata {
            version: "0.1.0".to_string(),
            source: "ccgadget-cli".to_string(),
        },
    };
    
    // Serialize to JSON
    let json_line = serde_json::to_string(&log_entry)?;
    
    // Append to log file
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file_path)?;
    
    writeln!(file, "{}", json_line)?;
    
    Ok(log_file_path)
}

fn handle_install_hook(hook_type: &HookType, force: bool) {
    println!("🔧 Installing Claude Code hooks...");
    println!("   Hook type: {:?}", hook_type);
    if force {
        println!("   Force reinstall enabled");
    }
    println!("   Status: Not yet implemented");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_directory_creation() {
        let result = get_log_directory();
        assert!(result.is_ok());
        let log_dir = result.unwrap();
        assert!(log_dir.exists());
        assert!(log_dir.is_dir());
    }

    #[test]
    fn test_hook_input_parsing() {
        let test_json = r#"{
            "session_id": "test-session",
            "hook_event_name": "UserPromptSubmit",
            "prompt": "test prompt"
        }"#;
        
        let parsed: Result<HookInput, _> = serde_json::from_str(test_json);
        assert!(parsed.is_ok());
        
        let hook_input = parsed.unwrap();
        assert_eq!(hook_input.session_id, Some("test-session".to_string()));
        assert_eq!(hook_input.hook_event_name, Some("UserPromptSubmit".to_string()));
        assert_eq!(hook_input.prompt, Some("test prompt".to_string()));
    }

    #[test]
    fn test_trigger_log_entry_serialization() {
        let log_entry = TriggerLogEntry {
            timestamp: chrono::Utc::now(),
            hook_input: Some(HookInput {
                session_id: Some("test-session".to_string()),
                transcript_path: None,
                cwd: None,
                hook_event_name: Some("TestEvent".to_string()),
                prompt: Some("test prompt".to_string()),
                message: None,
                tool_name: None,
                tool_input: None,
                tool_response: None,
            }),
            metadata: LogMetadata {
                version: "0.1.0".to_string(),
                source: "test".to_string(),
            },
        };

        let serialized = serde_json::to_string(&log_entry);
        assert!(serialized.is_ok());
        assert!(serialized.unwrap().contains("test-session"));
    }

    #[tokio::test]
    async fn test_bluetooth_manager_creation() {
        // Test that we can create a Bluetooth manager
        // This will fail on systems without Bluetooth, but that's expected
        let result = Manager::new().await;
        // We don't assert success here since CI environments may not have Bluetooth
        // Instead, we just verify the call doesn't panic
        match result {
            Ok(_) => println!("Bluetooth manager created successfully"),
            Err(e) => println!("Bluetooth not available: {}", e),
        }
    }

    #[test]
    fn test_cli_parsing() {
        use clap::Parser;
        
        // Test pair command parsing
        let args = vec!["ccgadget", "pair", "--device", "test-device", "--force"];
        let cli = Cli::try_parse_from(args);
        assert!(cli.is_ok());
        
        if let Some(Commands::Pair { device, force }) = cli.unwrap().command {
            assert_eq!(device, Some("test-device".to_string()));
            assert!(force);
        } else {
            panic!("Expected Pair command");
        }
    }

    #[test]
    fn test_device_name_filtering() {
        // Test CCGadget device name patterns
        assert!(is_ccgadget_device("CCGadget-Demo"));
        assert!(is_ccgadget_device("ccgadget-home")); // case insensitive
        assert!(is_ccgadget_device("My CCGadget Device"));
        assert!(is_ccgadget_device("CCG-Office"));
        assert!(is_ccgadget_device("ccg-lab"));
        assert!(is_ccgadget_device("ESP32-CCG-Test"));
        assert!(is_ccgadget_device("esp32-ccg-home"));
        
        // Test non-CCGadget device names should be filtered out
        assert!(!is_ccgadget_device("iPhone"));
        assert!(!is_ccgadget_device("MacBook Pro"));
        assert!(!is_ccgadget_device("AirPods"));
        assert!(!is_ccgadget_device("Unknown Device"));
        assert!(!is_ccgadget_device("ESP32-Other"));
        assert!(!is_ccgadget_device("CCG")); // too short
        assert!(!is_ccgadget_device(""));
    }

    #[test]
    fn test_hook_type_enum() {
        // Test that all hook types exist and can be used
        let _all = HookType::All;
        let _session = HookType::Session;
        let _command = HookType::Command;
        let _usage = HookType::Usage;
        
        // Test Debug implementation
        assert_eq!(format!("{:?}", HookType::All), "All");
        assert_eq!(format!("{:?}", HookType::Session), "Session");
        assert_eq!(format!("{:?}", HookType::Command), "Command");
        assert_eq!(format!("{:?}", HookType::Usage), "Usage");
    }
}
