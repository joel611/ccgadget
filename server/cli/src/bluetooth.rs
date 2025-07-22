use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Adapter, Manager, Peripheral};
use log::{error, info};
use std::error::Error;
use std::time::Duration;
use tokio::time;

pub async fn pair_device(mac: Option<String>) -> Result<(), Box<dyn Error>> {
    // Initialize Bluetooth manager
    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;
    let adapter = adapters
        .into_iter()
        .next()
        .ok_or("No Bluetooth adapters found")?;

    // Start scanning for devices
    info!("Scanning for Bluetooth devices...");
    adapter
        .start_scan(ScanFilter::default())
        .await
        .map_err(|e| format!("Failed to start scan: {}", e))?;

    // Scan for 10 seconds
    time::sleep(Duration::from_secs(10)).await;

    // Get discovered devices
    let peripherals = adapter.peripherals().await?;
    if peripherals.is_empty() {
        error!("No Bluetooth devices found. Ensure the target device is discoverable.");
        return Err("No devices found".into());
    }

    let mut target_peripheral: Option<Peripheral> = None;

    // If a MAC address is provided, find the matching device
    if let Some(mac_addr) = mac {
        for peripheral in peripherals.iter() {
            if let Ok(Some(addr)) = peripheral.address().await {
                if addr.to_string().to_uppercase() == mac_addr.to_uppercase() {
                    target_peripheral = Some(peripheral.clone());
                    break;
                }
            }
        }
        if target_peripheral.is_none() {
            error!("Device with MAC {} not found.", mac_addr);
            return Err("Specified device not found".into());
        }
    } else {
        // List all discovered devices
        info!("Discovered Bluetooth devices:");
        for (i, peripheral) in peripherals.iter().enumerate() {
            let properties = peripheral.properties().await?.unwrap_or_default();
            let name = properties.local_name.unwrap_or("Unknown".to_string());
            let addr = peripheral
                .address()
                .await
                .map(|a| a.to_string())
                .unwrap_or("Unknown".to_string());
            info!("{}. {} ({})", i + 1, name, addr);
        }

        // For simplicity, select the first device (modify to allow user input)
        target_peripheral = peripherals.into_iter().next();
    }

    let peripheral = target_peripheral.ok_or("No target device selected")?;
    let properties = peripheral.properties().await?.unwrap_or_default();
    let name = properties.local_name.unwrap_or("Unknown".to_string());
    let addr = peripheral
        .address()
        .await
        .map(|a| a.to_string())
        .unwrap_or("Unknown".to_string());

    // Attempt to pair
    info!("Attempting to pair with {} ({})...", name, addr);
    peripheral
        .pair()
        .await
        .map_err(|e| format!("Pairing failed: {}", e))?;

    info!("Successfully paired with {} ({})", name, addr);

    // Optional: Connect to the device (if required for ccusage data transmission)
    peripheral
        .connect()
        .await
        .map_err(|e| format!("Connection failed: {}", e))?;
    info!("Connected to {} ({})", name, addr);

    Ok(())
}
