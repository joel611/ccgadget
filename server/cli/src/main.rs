use clap::{CommandFactory, Parser, Subcommand};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::path::PathBuf;

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

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Pair { device, force }) => {
            handle_pair(device.as_deref(), *force);
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

fn handle_pair(device: Option<&str>, force: bool) {
    println!("🔵 Pairing with CCGadget device...");
    if let Some(device_name) = device {
        println!("   Target device: {}", device_name);
    } else {
        println!("   Scanning for nearby CCGadget devices...");
    }
    if force {
        println!("   Force pairing enabled");
    }
    println!("   Status: Not yet implemented");
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
