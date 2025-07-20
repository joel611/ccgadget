# 1. Overview
Develop a smart, IoT-enabled gadget, inspired by the Nest Thermostat, to monitor and display real-time usage and status of Claude code execution. The device will provide developers with an intuitive, glanceable interface to track Claude code session metrics (e.g., session countdown, costs, and status) on a circular 466x466 AMOLED touch display. The gadget will connect to a server via Bluetooth or 2.4GHz Wi-Fi, using Rust for both client and server software to ensure performance and safety.


# 2. Features
## 2.1 Core Features
- Real-Time Claude Code Monitoring:
  - Display current Claude code status: “Thinking” (processing) or “Waiting for User Instructions” (idle).
  - Data fetched via server hooks or API calls.
- Usage Metrics Display:
  - Session Countdown: Circular progress bar showing remaining session time (based on ccusage data, assumed max 600 seconds).
  - Cost Tracking: Real-time display of Claude code usage costs (e.g., API credits or dollars).
- Wireless Connectivity:
  - Bluetooth: Low-latency, short-range communication (within 10 meters).
  - 2.4GHz Wi-Fi: Reliable, longer-range communication for local networks.
- Interactive Touch Interface:
  - Navigate between Status (status text), Usage (progress bar, cost), and Settings screens via touch gestures (e.g., tap to switch).
  - Acknowledge alerts or reset session timers (if supported by Claude).




# 2. System Requirements
## 2.1 Hardware Requirements
### 2.1.1 Client (Gadget)
- Microcontroller Board: ESP32-S3R8
  - Dual-core processor with 8MB PSRAM for efficient processing.
  - Built-in support for Bluetooth 5.0 and 2.4GHz Wi-Fi.
  - GPIO pins for interfacing with the display and other peripherals.
- Display: 466x466 AMOLED Touch Circular Display
  - High-resolution, vibrant display for clear visualization.
  - Capacitive touch for user interaction (e.g., navigating menus or acknowledging status updates).
- Connectivity:
  - Bluetooth: For low-power, short-range communication with the server.
  - 2.4GHz Wi-Fi: For robust, longer-range communication and OTA updates.
- Power Supply:
  - Rechargeable Li-Po battery (e.g., 3.7V, 1000mAh) for portability.
  - USB-C port for charging and debugging.

---

# 3. Technical Requirements
## 3.1 Hardware Requirements
### 3.1.1 Client (Gadget)
[ESP32-S3-Touch-amoled-1.43](https://www.waveshare.net/wiki/ESP32-S3-Touch-AMOLED-1.43)
- Microcontroller: ESP32-S3R8
  - Dual-core Xtensa LX7 processor, 8MB PSRAM.
  - Built-in Bluetooth 5.0 and 2.4GHz Wi-Fi.
  - GPIO pins for SPI (display) and I2C (touch).
- Display: 466x466 AMOLED Touch Circular Display
  - High-resolution, vibrant display for clear visuals.
  - Capacitive touch for user interaction.
  - Driver: 
    - Display (QSPI): SH8601/CO5300, 
    - Touch (I2C): FT3168 
- Power Supply:
  - Rechargeable Li-Po battery (3.7V, 1000mAh) for portability.
  - USB-C port for charging and debugging.
- Chips:
  - PCF85063: RTC chip
  - QMI8658c: reading and printing accelerometer data, gyroscope data, and temperature data

- AMOLED Definition

| AMOLED pin |	ESP32-S3 pin |	Description |
|---|---|---|
|QSPI_CS |	GPIO 9 | QSPI chip selection
|QSPI_CLK | GPIO 10 | QSPI clock pin
|QSPI_D0 | GPIO 11 | QSPI D0 data
|QSPI_D1 | GPIO 12 | QSPI D1 data
|QSPI_D2 | GPIO 13 | QSPI D2 data
|QSPI_D3 | GPIO 14 | QSPI D3 data
|AMOLED_RESET | GPIO 21 |	AMOLED reset pin
|AMOLED_EN | GPIO 42 |	AMOLED enable pin
|TP_SDA |	GPIO 47 |	TP I2C data pin
|TP_SCL |	GPIO 48 |	TP I2C clock pin


### 3.1.2 Server
  - Supported Platforms: macOS, Linux (e.g., Ubuntu), Windows 10/11.
  - Minimum Specs: 4GB RAM, dual-core CPU, 10GB free storage.
  - Recommended Specs: 8GB RAM, quad-core CPU for smooth Claude execution.
  - Connectivity: Wi-Fi or Bluetooth module compatible with ESP32-S3R8.



## 3.2 Software Requirements
### 3.2.1 Client
- Programming Language: Rust (`no_std` for embedded constraints).
- Framework: `esp32s3-hal` for hardware abstraction.
- UI Library: `embedded-graphics` for rendering progress bars, text, and basic graphics (simpler for Rust beginners than `lvgl-sys`).
- Dependencies:
  - `serde_json_core` for JSON parsing in `no_std`.
  - `esp-wifi` for Wi-Fi and Bluetooth connectivity.
  - `esp-alloc` for heap allocation.
- Tools:
  - `esp-rs/esp-generate` for project setup.
  - `espflash` for flashing firmware.
  - Wokwi Simulator for testing without hardware.
- Features:
  - Initialize AMOLED display via SPI and touch controller via I2C.
  - Connect to server via Wi-Fi (TCP) or Bluetooth (BLE).
  - Parse and display JSON data: {"status": String, "session_time": u32, "cost": f32}.
  - Handle touch input for screen navigation.
### 3.2.2 Server
- Programming Language: Rust.
- Framework: `tokio` for async networking, `eframe` for cross-platform GUI.
- Dependencies:
  - `reqwest` for Claude API calls (assumed HTTP-based).
  - `serde` for JSON serialization.
  - `bluer` for Bluetooth communication.
- Integration:
  - Interface with Claude code via HTTP API or external ccusage process (assumed JSON output).
  - GUI for configuration (e.g., select Wi-Fi/Bluetooth, view logs).
- Supported Platforms: macOS, Linux, Windows.
- Tools: Cargo for dependency management, `espflash` for client flashing support.










## Installation Step
1. Install `rustup`
2. `cargo install espup espflash probe-rs`
3. `cargo binstall probe-rs-tools`
