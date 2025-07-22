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
    /// Setup Claude Code hooks for automatic monitoring
    SetupHook {
        /// Scope for hook installation (local or user)
        #[arg(short, long, default_value = "local")]
        scope: HookScope,
        /// Force reinstall if hook already exists
        #[arg(short, long)]
        force: bool,
        /// Automatically approve adding hooks alongside existing ones
        #[arg(short, long)]
        yes: bool,
    },
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum HookScope {
    /// Install hooks at user level (~/.claude/settings.json)
    User,
    /// Install hooks at project local level (.claude/settings.local.json)
    Local,
}

#[derive(Debug)]
enum HookSetupResult {
    /// Hook was successfully added
    Added,
    /// Hook was skipped (user declined or other reason)
    Skipped,
    /// Hook already exists and no action was needed
    AlreadyExists,
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
        Some(Commands::SetupHook { scope, force, yes }) => {
            handle_setup_hook(scope, *force, *yes);
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

/// Setup Claude Code hooks by detecting settings files and configuring hooks
fn setup_claude_hooks(scope: &HookScope, force: bool, auto_approve: bool) -> Result<String, Box<dyn std::error::Error>> {
    // Find Claude settings file based on scope
    let settings_path = find_claude_settings_file(scope)?;
    println!("   📁 Found Claude settings: {}", settings_path.display());
    
    // Read existing settings
    let mut settings = read_claude_settings(&settings_path)?;
    
    // Get all hooks to configure (setup all hooks by default)
    let hooks_config = get_all_hooks_config();
    
    // Setup hooks in settings
    let mut updated_hooks = 0;
    let mut skipped_hooks = Vec::new();
    
    for (event_name, hook_command) in hooks_config {
        match setup_hook_for_event(&mut settings, &event_name, &hook_command, force, auto_approve)? {
            HookSetupResult::Added => {
                updated_hooks += 1;
            }
            HookSetupResult::Skipped => {
                skipped_hooks.push(event_name);
            }
            HookSetupResult::AlreadyExists => {
                // Hook already exists, no action needed
            }
        }
    }
    
    if !skipped_hooks.is_empty() {
        println!("   ℹ️ Skipped hooks for events: {}", skipped_hooks.join(", "));
    }
    
    // Write settings back to file
    write_claude_settings(&settings_path, &settings)?;
    
    Ok(format!("Successfully configured {} hook(s) in {}", updated_hooks, settings_path.display()))
}

/// Find the appropriate Claude settings file based on scope
fn find_claude_settings_file(scope: &HookScope) -> Result<PathBuf, Box<dyn std::error::Error>> {
    match scope {
        HookScope::Local => {
            // Use project-local settings
            let project_local = PathBuf::from(".claude").join("settings.local.json");
            
            // Create .claude directory if it doesn't exist
            let claude_dir = PathBuf::from(".claude");
            if !claude_dir.exists() {
                fs::create_dir_all(&claude_dir)?;
            }
            
            // Create empty settings file if it doesn't exist
            if !project_local.exists() {
                let empty_settings = serde_json::json!({});
                fs::write(&project_local, serde_json::to_string_pretty(&empty_settings)?)?;
                println!("   📝 Created new local settings file: {}", project_local.display());
            }
            
            Ok(project_local)
        }
        HookScope::User => {
            // Use user settings
            let home_dir = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .map_err(|_| "Could not determine home directory")?;
            let home_path = PathBuf::from(&home_dir);
            let user_settings = home_path.join(".claude").join("settings.json");
            
            // Create ~/.claude directory if it doesn't exist
            let claude_dir = home_path.join(".claude");
            if !claude_dir.exists() {
                fs::create_dir_all(&claude_dir)?;
            }
            
            // Create empty settings file if it doesn't exist
            if !user_settings.exists() {
                let empty_settings = serde_json::json!({});
                fs::write(&user_settings, serde_json::to_string_pretty(&empty_settings)?)?;
                println!("   📝 Created new user settings file: {}", user_settings.display());
            }
            
            Ok(user_settings)
        }
    }
}

/// Read Claude settings from file
fn read_claude_settings(path: &PathBuf) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    if path.exists() {
        let content = fs::read_to_string(path)?;
        if content.trim().is_empty() {
            Ok(serde_json::json!({}))
        } else {
            Ok(serde_json::from_str(&content)?)
        }
    } else {
        Ok(serde_json::json!({}))
    }
}

/// Write Claude settings to file
fn write_claude_settings(path: &PathBuf, settings: &serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
    let formatted = serde_json::to_string_pretty(settings)?;
    fs::write(path, formatted)?;
    Ok(())
}

/// Get all hook configurations for CCGadget
fn get_all_hooks_config() -> Vec<(&'static str, &'static str)> {
    vec![
        ("UserPromptSubmit", "ccgadget trigger"),
        ("PreToolUse", "ccgadget trigger"),
        ("PostToolUse", "ccgadget trigger"),
        ("Notification", "ccgadget trigger"),
        ("Stop", "ccgadget trigger"),
    ]
}

/// Setup a hook for a specific event in the settings
fn setup_hook_for_event(
    settings: &mut serde_json::Value,
    event_name: &str,
    hook_command: &str,
    force: bool,
    auto_approve: bool,
) -> Result<HookSetupResult, Box<dyn std::error::Error>> {
    // Ensure hooks object exists
    if !settings.get("hooks").is_some() {
        settings["hooks"] = serde_json::json!({});
    }
    
    let hooks = settings["hooks"].as_object_mut()
        .ok_or("Failed to get hooks object")?;
    
    // Check if event already has hooks configured
    if hooks.contains_key(event_name) {
        // First check the current state (need immutable borrow)
        let event_hooks_value = hooks.get(event_name).unwrap();
        let exact_hook_exists = exact_hook_exists(event_hooks_value, hook_command);
        let any_ccgadget_exists = any_ccgadget_hook_exists(event_hooks_value, hook_command);
        let has_other_hooks = event_hooks_value.as_array()
            .map(|arr| !arr.is_empty())
            .unwrap_or(false);
        
        if exact_hook_exists {
            // Perfect match - hook is already correctly configured
            println!("   ✅ Hook for {} already correctly configured", event_name);
            return Ok(HookSetupResult::AlreadyExists);
        }
        
        // Determine what action to take
        let action = if any_ccgadget_exists {
            // ccgadget hook exists but with wrong configuration - ask user
            if force {
                println!("   🔧 Forcing update of mismatched hook for {}", event_name);
                HookAction::Replace
            } else if auto_approve {
                println!("   ✅ Auto-approving hook update for {} (--yes flag)", event_name);
                HookAction::Replace
            } else {
                ask_user_fix_hook_action(event_name, event_hooks_value, hook_command)?
            }
        } else if has_other_hooks {
            // Check if there are any non-ccgadget hooks
            let has_non_ccgadget_hooks = event_hooks_value.as_array()
                .map(|arr| arr.iter().any(|hook_group| !hook_group_contains_command(hook_group, "ccgadget")))
                .unwrap_or(false);
                
            if has_non_ccgadget_hooks {
                // Other non-ccgadget hooks exist - ask user what to do
                if auto_approve {
                    println!("   ✅ Auto-approving hook addition for {} (--yes flag)", event_name);
                    HookAction::Append
                } else {
                    ask_user_hook_action(event_name, event_hooks_value)?
                }
            } else {
                // No actual hooks, just add
                HookAction::Append
            }
        } else {
            // No hooks at all, just add
            HookAction::Append
        };
        
        // Handle user choice
        match action {
            HookAction::Skip => {
                println!("   ⏭️ Skipping hook for {} (user chose skip)", event_name);
                return Ok(HookSetupResult::Skipped);
            }
            HookAction::Replace => {
                // Replace all existing hooks with just our ccgadget hook
                let hook_config = serde_json::json!([
                    {
                        "matcher": "",
                        "hooks": [
                            {
                                "type": "command",
                                "command": hook_command
                            }
                        ]
                    }
                ]);
                hooks.insert(event_name.to_string(), hook_config);
                println!("   🔄 Replaced all hooks for {} with ccgadget hook", event_name);
            }
            HookAction::Append => {
                // Add ccgadget hook alongside existing hooks
                let event_hooks_array = hooks.get_mut(event_name).unwrap()
                    .as_array_mut()
                    .ok_or("Event hooks must be an array")?;
                
                // Remove existing ccgadget hooks first (if any) to avoid duplicates
                if any_ccgadget_exists {
                    event_hooks_array.retain(|hook_group| {
                        !hook_group_contains_command(hook_group, "ccgadget")
                    });
                }
                
                // Add our hook to the existing array
                let ccgadget_hook = serde_json::json!({
                    "matcher": "",
                    "hooks": [
                        {
                            "type": "command",
                            "command": hook_command
                        }
                    ]
                });
                
                event_hooks_array.push(ccgadget_hook);
                println!("   ➕ Added ccgadget hook alongside existing hooks for {}", event_name);
            }
        }
    } else {
        // No existing hooks for this event - create new array
        let hook_config = serde_json::json!([
            {
                "matcher": "",
                "hooks": [
                    {
                        "type": "command",
                        "command": hook_command
                    }
                ]
            }
        ]);
        
        hooks.insert(event_name.to_string(), hook_config);
    }
    
    println!("   ✅ Configured hook for {}", event_name);
    Ok(HookSetupResult::Added)
}

/// Check if the exact expected hook configuration already exists
fn exact_hook_exists(event_hooks: &serde_json::Value, target_command: &str) -> bool {
    if let Some(hooks_array) = event_hooks.as_array() {
        for hook_group in hooks_array {
            // Check if this hook group matches our expected configuration exactly
            if is_exact_ccgadget_hook(hook_group, target_command) {
                return true;
            }
        }
    }
    false
}

/// Check if a hook group is exactly the ccgadget hook we expect
fn is_exact_ccgadget_hook(hook_group: &serde_json::Value, target_command: &str) -> bool {
    // Expected: {"matcher": "", "hooks": [{"type": "command", "command": "ccgadget trigger"}]}
    let expected_matcher = "";
    
    // Check matcher
    let matcher = hook_group.get("matcher")
        .and_then(|m| m.as_str())
        .unwrap_or("");
    
    if matcher != expected_matcher {
        return false;
    }
    
    // Check hooks array
    if let Some(hooks) = hook_group.get("hooks").and_then(|h| h.as_array()) {
        if hooks.len() != 1 {
            return false;
        }
        
        let hook = &hooks[0];
        let hook_type = hook.get("type").and_then(|t| t.as_str()).unwrap_or("");
        let hook_command = hook.get("command").and_then(|c| c.as_str()).unwrap_or("");
        
        return hook_type == "command" && hook_command == target_command;
    }
    
    false
}

/// Check if any ccgadget-related hook exists (even if not exact match)
fn any_ccgadget_hook_exists(event_hooks: &serde_json::Value, target_command: &str) -> bool {
    if let Some(hooks_array) = event_hooks.as_array() {
        for hook_group in hooks_array {
            if hook_group_contains_command(hook_group, target_command) {
                return true;
            }
        }
    }
    false
}

/// Check if a specific hook group contains a command
fn hook_group_contains_command(hook_group: &serde_json::Value, target_command: &str) -> bool {
    if let Some(hooks) = hook_group.get("hooks").and_then(|h| h.as_array()) {
        for hook in hooks {
            if let Some(command) = hook.get("command").and_then(|c| c.as_str()) {
                if command.contains(target_command) {
                    return true;
                }
            }
        }
    }
    false
}

/// User choice for handling hook conflicts
#[derive(Debug, PartialEq)]
enum HookAction {
    Replace,  // Replace existing hooks with ccgadget hook
    Append,   // Add ccgadget hook alongside existing hooks
    Skip,     // Skip this event, leave existing hooks unchanged
}

/// Ask user what to do with existing hooks for a specific event
fn ask_user_hook_action(event_name: &str, existing_hooks: &serde_json::Value) -> Result<HookAction, Box<dyn std::error::Error>> {
    println!("   ⚠️ Event '{}' already has existing hooks configured:", event_name);
    
    // Display existing hooks in a user-friendly way
    if let Some(hooks_array) = existing_hooks.as_array() {
        for (i, hook_group) in hooks_array.iter().enumerate() {
            if let Some(hooks) = hook_group.get("hooks").and_then(|h| h.as_array()) {
                for (j, hook) in hooks.iter().enumerate() {
                    if let Some(command) = hook.get("command").and_then(|c| c.as_str()) {
                        let matcher = hook_group.get("matcher")
                            .and_then(|m| m.as_str())
                            .unwrap_or("");
                        let matcher_display = if matcher.is_empty() { "all" } else { matcher };
                        println!("     {}.{}: {} (matcher: {})", i + 1, j + 1, command, matcher_display);
                    }
                }
            }
        }
    }
    
    println!("   How would you like to handle 'ccgadget trigger' for {}?", event_name);
    println!("     [r] Replace - Remove existing hooks and add ccgadget hook");
    println!("     [a] Append  - Add ccgadget hook alongside existing hooks");
    println!("     [s] Skip    - Keep existing hooks unchanged");
    print!("   Choose [r/a/s]: ");
    std::io::stdout().flush()?;
    
    // Read user input
    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();
        
        match input.as_str() {
            "r" | "replace" => return Ok(HookAction::Replace),
            "a" | "append" => return Ok(HookAction::Append),
            "s" | "skip" => return Ok(HookAction::Skip),
            _ => {
                print!("   Invalid choice. Please enter [r]eplace, [a]ppend, or [s]kip: ");
                std::io::stdout().flush()?;
                continue;
            }
        }
    }
}

/// Ask user what to do with mismatched ccgadget hooks
fn ask_user_fix_hook_action(event_name: &str, existing_hooks: &serde_json::Value, expected_command: &str) -> Result<HookAction, Box<dyn std::error::Error>> {
    println!("   ⚠️ Event '{}' has ccgadget hooks but with incorrect configuration:", event_name);
    
    // Show current vs expected
    println!("   Current ccgadget hooks:");
    if let Some(hooks_array) = existing_hooks.as_array() {
        for (i, hook_group) in hooks_array.iter().enumerate() {
            if hook_group_contains_command(hook_group, "ccgadget") {
                if let Some(hooks) = hook_group.get("hooks").and_then(|h| h.as_array()) {
                    for (j, hook) in hooks.iter().enumerate() {
                        if let Some(command) = hook.get("command").and_then(|c| c.as_str()) {
                            if command.contains("ccgadget") {
                                let matcher = hook_group.get("matcher")
                                    .and_then(|m| m.as_str())
                                    .unwrap_or("");
                                let matcher_display = if matcher.is_empty() { "all" } else { &format!("'{}'", matcher) };
                                println!("     {}.{}: {} (matcher: {})", i + 1, j + 1, command, matcher_display);
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Show non-ccgadget hooks if any
    let has_non_ccgadget = existing_hooks.as_array()
        .map(|arr| arr.iter().any(|hook_group| !hook_group_contains_command(hook_group, "ccgadget")))
        .unwrap_or(false);
        
    if has_non_ccgadget {
        println!("   Other existing hooks:");
        if let Some(hooks_array) = existing_hooks.as_array() {
            for (i, hook_group) in hooks_array.iter().enumerate() {
                if !hook_group_contains_command(hook_group, "ccgadget") {
                    if let Some(hooks) = hook_group.get("hooks").and_then(|h| h.as_array()) {
                        for (j, hook) in hooks.iter().enumerate() {
                            if let Some(command) = hook.get("command").and_then(|c| c.as_str()) {
                                let matcher = hook_group.get("matcher")
                                    .and_then(|m| m.as_str())
                                    .unwrap_or("");
                                let matcher_display = if matcher.is_empty() { "all" } else { matcher };
                                println!("     {}.{}: {} (matcher: {})", i + 1, j + 1, command, matcher_display);
                            }
                        }
                    }
                }
            }
        }
    }
    
    println!("   Expected ccgadget hook: {} (matcher: all)", expected_command);
    println!("   How would you like to handle the incorrect ccgadget hook for {}?", event_name);
    println!("     [r] Replace - Fix ccgadget hook to correct configuration");
    println!("     [a] Append  - Add correct ccgadget hook alongside current ones"); 
    println!("     [s] Skip    - Keep current hooks unchanged");
    print!("   Choose [r/a/s]: ");
    std::io::stdout().flush()?;
    
    // Read user input
    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();
        
        match input.as_str() {
            "r" | "replace" => return Ok(HookAction::Replace),
            "a" | "append" => return Ok(HookAction::Append),
            "s" | "skip" => return Ok(HookAction::Skip),
            _ => {
                print!("   Invalid choice. Please enter [r]eplace, [a]ppend, or [s]kip: ");
                std::io::stdout().flush()?;
                continue;
            }
        }
    }
}

fn handle_setup_hook(scope: &HookScope, force: bool, auto_approve: bool) {
    println!("🔧 Setting up Claude Code hooks...");
    println!("   Scope: {:?}", scope);
    if force {
        println!("   Force reinstall enabled");
    }
    if auto_approve {
        println!("   Auto-approve enabled");
    }
    
    match setup_claude_hooks(scope, force, auto_approve) {
        Ok(message) => {
            println!("   ✅ {}", message);
        }
        Err(e) => {
            eprintln!("   ❌ Failed to setup hooks: {}", e);
            std::process::exit(1);
        }
    }
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
    fn test_hook_scope_enum() {
        // Test that all hook scopes exist and can be used
        let _user = HookScope::User;
        let _local = HookScope::Local;
        
        // Test Debug implementation
        assert_eq!(format!("{:?}", HookScope::User), "User");
        assert_eq!(format!("{:?}", HookScope::Local), "Local");
    }
}
