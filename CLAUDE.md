# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

CCGadget is an IoT-enabled hardware gadget that monitors and displays real-time Claude Code usage metrics. It consists of:

- **Client**: ESP32-S3 microcontroller with embedded Rust firmware
- **Server**: Cross-platform CLI tool (this directory) for desktop systems

The device displays Claude code session status, countdown timers, and costs on a circular AMOLED display, inspired by the Nest Thermostat design.

## Development Commands

Currently this is a new Rust CLI project in early development. Basic commands:

```bash
# Build the CLI tool
cargo build

# Run the CLI tool
cargo run

# Run tests
cargo test

# Run with specific features
cargo run --features <feature-name>
```

## Architecture

### Current State

- **Language**: Standard Rust (not embedded like the client)
- **Target**: Cross-platform CLI tool for macOS/Linux/Windows
- **Integration**: Planned integration with Claude Code hooks and ccusage tool

### Planned CLI Commands

- `ccgadget pair` - Bluetooth device pairing with ESP32 client
- `ccgadget start` - Background daemon to fetch and send usage data
- `ccgadget trigger` - Claude Code hook integration for real-time events
- `ccgadget setup-hook` - Setup Claude Code hooks helper

### System Integration

- **Claude Code Hooks**: Integration planned for real-time session monitoring
- **ccusage Tool**: External dependency for usage data collection
- **Bluetooth/Wi-Fi**: Communication with ESP32-S3 hardware client
- **Background Daemon**: Continuous monitoring and data transmission

### Project Structure

```
server/cli/          # This directory - Desktop CLI tool
├── src/            # Rust source code
├── Cargo.toml      # Dependencies and project config
└── target/         # Build artifacts

Related directories:
../../client/        # ESP32 embedded firmware (separate ecosystem)
../../README.md      # Main project documentation
```

## Hardware Context

When working on this CLI tool, understand it communicates with:

- **ESP32-S3R8**: Dual-core microcontroller with 8MB PSRAM
- **466x466 AMOLED Display**: Circular touch screen for metrics display
- **Connectivity**: Bluetooth 5.0 + 2.4GHz Wi-Fi
- **Additional Sensors**: RTC chip (PCF85063), accelerometer/gyroscope (QMI8658c)

The CLI serves as the bridge between Claude Code usage data and the physical display device.

## Distribution

Planned distribution via Homebrew: `brew install ccgadget`

## Development Notes

- This is a standard Rust project (not embedded like the client firmware)
- Focus on cross-platform compatibility for desktop systems
- Integration with external tools (Claude Code, ccusage) is key to functionality
- Bluetooth/networking code will be central to device communication

## Code Style

- every function must include a concise, purpose-driven docstring
