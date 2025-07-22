# CCGadget CLI

Desktop command-line tool for [CCGadget](../../README.md) - an IoT-enabled hardware gadget that monitors and displays real-time Claude Code usage metrics on a circular AMOLED display.

## Overview

This CLI tool serves as the bridge between your Claude Code sessions and the CCGadget hardware device, enabling:

- 🔗 **Bluetooth Pairing** - Connect to ESP32-S3 hardware via Bluetooth LE
- 📊 **Real-time Monitoring** - Stream Claude Code usage data to the display
- ⚡ **Hook Integration** - Automatic event capture via Claude Code hooks
- 🎯 **Background Service** - Continuous monitoring daemon

## Installation

### From Source (Current)

```bash
git clone <repository-url>
cd ccgadget/server/cli
cargo build --release
cargo install --path .
```

### Homebrew (Planned)

```bash
brew install ccgadget
```

## Quick Start

1. **Build the CLI**:

   ```bash
   ./build.sh
   ```

2. **Pair with your CCGadget device**:

   ```bash
   ccgadget pair
   ```

3. **Install Claude Code hooks**:

   ```bash
   ccgadget install-hook all
   ```

4. **Start monitoring daemon**:
   ```bash
   ccgadget start
   ```

## Commands

### `ccgadget pair`

Pair with CCGadget device via Bluetooth LE scanning.

```bash
# Interactive device selection
ccgadget pair

# Pair with specific device
ccgadget pair --device "CCGadget-ABC123"
ccgadget pair --device "AA:BB:CC:DD:EE:FF"

# Force re-pairing
ccgadget pair --force
```

**Features:**

- 10-second Bluetooth scan
- Interactive device selection with signal strength
- Support for device name or MAC address
- Service discovery and connection verification

### `ccgadget start`

Start background daemon to monitor Claude Code usage.

```bash
# Background daemon (default)
ccgadget start

# Foreground mode for debugging
ccgadget start --foreground

# Custom update interval
ccgadget start --interval 15
```

### `ccgadget trigger`

Process Claude Code hook events (used internally by hooks).

```bash
# Triggered automatically by hooks
echo '{"session_id":"abc","hook_event_name":"UserPromptSubmit"}' | ccgadget trigger
```

### `ccgadget setup-hook`

Setup Claude Code hooks helper.

```bash
# Setup (detault to local level)
ccgadget setup-hook

# Setup on different levels
ccgadget setup-hook -s user
ccgadget setup-hook -s local

```

## Development

### Building

```bash
# Development build
cargo build

# Optimized release build
./build.sh

# Install locally
cargo install --path .
```

### Testing

```bash
# Complete test suite
./test.sh

# Individual test categories
cargo test --bin ccgadget           # Unit tests (6 tests)
cargo test --test integration_tests # Integration tests (7 tests)
cargo test --test bluetooth_tests   # Bluetooth tests (6 tests)

# Hook functionality tests
./test_all_hooks.sh
```

**Test Coverage:**

- ✅ CLI argument parsing and validation
- ✅ JSON hook data processing and logging
- ✅ Bluetooth manager initialization
- ✅ Binary functionality verification
- ✅ Command help systems
- ✅ Log directory creation and management

### Project Structure

```
server/cli/
├── src/
│   ├── main.rs          # Main CLI application
│   └── bluetooth.rs     # Bluetooth pairing functionality
├── tests/               # Integration tests
├── test_data/          # Sample hook data for testing
├── build.sh            # Release build script
├── test.sh             # Unified test runner
└── test_all_hooks.sh   # Hook functionality tests
```

## Hook Integration

### Automatic Installation

```bash
ccgadget install-hook all
```

### Manual Hook Configuration

Add to your Claude Code hooks configuration:

```json
{
  "hooks": {
    "UserPromptSubmit": [
      {
        "matcher": ".*",
        "hooks": [
          {
            "type": "command",
            "command": "ccgadget trigger"
          }
        ]
      }
    ],
    "PostToolUse": [
      {
        "matcher": ".*",
        "hooks": [
          {
            "type": "command",
            "command": "ccgadget trigger"
          }
        ]
      }
    ]
  }
}
```

## Supported Platforms

- **macOS** (Intel & Apple Silicon)
- **Linux** (x86_64, ARM64)
- **Windows** (x86_64)

**Requirements:**

- Rust 1.70+ for building from source
- Bluetooth 5.0+ adapter for device pairing
- Claude Code for hook integration

## Hardware Compatibility

**CCGadget Device Specifications:**

- **MCU**: ESP32-S3R8 (dual-core, 8MB PSRAM)
- **Display**: 466x466 circular AMOLED with touch
- **Connectivity**: Bluetooth 5.0 LE + 2.4GHz Wi-Fi
- **Sensors**: RTC (PCF85063), IMU (QMI8658c)

## Troubleshooting

### Bluetooth Issues

```bash
# Check Bluetooth adapter
ccgadget pair --help

# Enable verbose logging
RUST_LOG=debug ccgadget pair
```

### Hook Issues

```bash
# Test hook functionality
./test_all_hooks.sh

# Check log output
tail -f ~/.ccgadget/logs/trigger-$(date +%Y-%m-%d).log
```

### Build Issues

```bash
# Clean and rebuild
cargo clean
./build.sh

# Check dependencies
cargo tree
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Run the test suite: `./test.sh`
5. Submit a pull request

## References

- [Claude Code Hooks Documentation](https://docs.anthropic.com/en/docs/claude-code/hooks)
- [ccusage - Claude Code Usage Tracker](https://github.com/ryoppippi/ccusage)
- [ESP32-S3 Datasheet](https://www.espressif.com/en/products/socs/esp32-s3)
- [Bluetooth LE Specification](https://www.bluetooth.com/specifications/bluetooth-core-specification/)

---

Built with ❤️ in Rust | Part of the CCGadget IoT ecosystem
