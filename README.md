[<img alt="github" src="https://img.shields.io/badge/github-rttfd/renik-37a8e0?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/rttfd/renik)
[<img alt="crates.io" src="https://img.shields.io/crates/v/renik.svg?style=for-the-badge&color=ff8b94&logo=rust" height="20">](https://crates.io/crates/renik)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-renik-bedc9c?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/renik)

![Dall-E generated renik image](https://raw.githubusercontent.com/rttfd/static/refs/heads/main/renik/renik.jpeg)

# Renik

Comprehensive embedded device configuration library for `no_std` environments.

Renik provides robust configuration structures for embedded devices, featuring complete Wi-Fi connectivity management, advanced Bluetooth device lifecycle management with finite state machine (FSM) support, and secure device identification. All structures are designed for `no_std` environments with `#[repr(C)]` memory layout for reliable serialization and persistent storage.

## Features

- **Wi-Fi Configuration**: Store and manage Wi-Fi network credentials with `WifiConfig`
- **Complete Bluetooth Management**: Full device lifecycle management including:
  - Individual device information with pairing data (`BluetoothDeviceInfo`)
  - Multi-device list management up to 10 devices (`BluetoothDeviceList`)
  - Real-time connection state tracking with FSM (`BluetoothConnectionState`)
  - Connection phase management with `BluetoothConnectionPhase` enum
  - Type-safe connection handles with validation (`ConnHandle`)
  - Low-level connection parameters (`BluetoothConnectionParams`)
  - Security and authentication data (`BluetoothSecurityInfo`)
- **Device Identity**: Secure device identification and authentication with `DeviceInfo`
- **Memory Safe**: Fixed-size buffers with length tracking prevent overflows
- **Serializable**: `#[repr(C)]` layout ensures consistent cross-platform serialization
- **Embedded Ready**: Full `no_std` compatibility with minimal dependencies
- **Production Ready**: Comprehensive error handling and validation
- **State Management**: Advanced finite state machine for Bluetooth connection phases

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
renik = "0.6.0"
```

### Wi-Fi Configuration

```rust
use renik::WifiConfig;

// Create a new Wi-Fi configuration
let config = WifiConfig::new(b"MyNetwork", b"password123")?;

// Validate and use
if config.is_valid() {
    println!("SSID: {:?}", config.get_ssid());
    println!("Password: {:?}", config.get_password());
}
```

### Bluetooth Device Management

#### Individual Device Configuration

```rust
use renik::BluetoothDeviceInfo;

// Create a Bluetooth device configuration
let mac_address = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC];
let mut device = BluetoothDeviceInfo::new(&mac_address, b"My Bluetooth Speaker")?;

// Set pairing information
device.set_pairing_key(b"audio_key_123")?;
device.set_class_of_device(&[0x04, 0x04, 0x24]); // Audio device

// Set device flags
device.add_flag(BluetoothDeviceInfo::FLAG_PAIRED);
device.add_flag(BluetoothDeviceInfo::FLAG_TRUSTED);
device.add_flag(BluetoothDeviceInfo::FLAG_AUDIO);
device.add_flag(BluetoothDeviceInfo::FLAG_AUTO_RECONNECT);

// Check device status
if device.is_paired() && device.supports_auto_reconnect() {
    println!("Device ready for automatic reconnection");
}
```

#### Managing Multiple Devices

```rust
use renik::{BluetoothDeviceInfo, BluetoothDeviceList};

// Create device list (supports up to 10 devices)
let mut device_list = BluetoothDeviceList::default();

// Add multiple devices
let speaker_mac = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC];
let speaker = BluetoothDeviceInfo::new(&speaker_mac, b"Speaker")?;

let mouse_mac = [0x98, 0x76, 0x54, 0x32, 0x10, 0xFE];
let mouse = BluetoothDeviceInfo::new(&mouse_mac, b"Mouse")?;

device_list.add_device(speaker)?;
device_list.add_device(mouse)?;

// Iterate through devices
for i in 0..device_list.len() {
    let device = device_list.get_device(i)?;
    if device.is_paired() {
        println!("Found paired device: {:?}", device.get_device_name());
    }
}
```

#### Connection State Management with FSM

```rust
use renik::{BluetoothConnectionState, BluetoothConnectionPhase, BluetoothDeviceInfo, ConnHandle};

// Track connection state with finite state machine
let mut connection_state = BluetoothConnectionState::default();

// Set remote device
let mac_addr = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC];
let device = BluetoothDeviceInfo::new(&mac_addr, b"Audio Device")?;
connection_state.set_remote_device(device);

// Advance through connection phases using FSM
assert!(connection_state.advance_to_phase(BluetoothConnectionPhase::Discovery));
assert!(connection_state.advance_to_phase(BluetoothConnectionPhase::Connecting));
assert!(connection_state.advance_to_phase(BluetoothConnectionPhase::Connected));

// Invalid transitions are rejected
assert!(!connection_state.advance_to_phase(BluetoothConnectionPhase::Ready)); // Invalid: must authenticate first

// Continue through authentication
assert!(connection_state.advance_to_phase(BluetoothConnectionPhase::Authenticating));
assert!(connection_state.advance_to_phase(BluetoothConnectionPhase::SettingUpEncryption));
assert!(connection_state.advance_to_phase(BluetoothConnectionPhase::FullyConnected));
assert!(connection_state.advance_to_phase(BluetoothConnectionPhase::Ready));

// Check current phase and state
let current_phase = connection_state.get_connection_phase();
println!("Current phase: {:?}", current_phase);
println!("Is connected: {}", current_phase.is_connected());
println!("Is secure: {}", current_phase.is_secure());
println!("Is ready: {}", current_phase.is_ready());

// Type-safe connection handles
let handle = ConnHandle::new(0x0001); // Validates range 0x0000-0x0EFF
connection_state.set_connection_handle(Some(handle));
println!("Connection handle: 0x{:04X}", handle.raw());
```

#### Connection Phase States

The Bluetooth FSM supports the following connection phases:

- **Idle**: Initial state, no connection attempt
- **Discovery**: Discovering available devices
- **Connecting**: Initiating connection to specific device
- **Connected**: Basic connection established (not authenticated)
- **Authenticating**: Authentication in progress
- **SettingUpEncryption**: Setting up encrypted communication
- **FullyConnected**: Connected, authenticated, and encrypted
- **ServiceDiscovery**: Discovering available services
- **Ready**: Connection ready for use
- **Maintaining**: Connection maintenance mode
- **Reconnecting**: Attempting to reconnect after connection loss
- **Failed**: Connection failed
- **Disconnecting**: Graceful disconnection in progress

### Device Information

```rust
use renik::DeviceInfo;

// Create device identity configuration
let device_info = DeviceInfo::new(
    b"RENIK-01JY1863M2V0S776",
    b"device_secret_key"
)?;

if device_info.is_valid() {
    println!("Hardware ID: {:?}", device_info.get_hardware_id());
}
```

### Advanced Bluetooth Features

#### Connection Parameters

```rust
use renik::{BluetoothConnectionParams, BluetoothDeviceInfo};

// Create connection parameters
let mut params = BluetoothConnectionParams::default();
params.connection_handle = 0x0001;
params.connection_interval = 24; // 30ms (24 * 1.25ms)
params.connection_latency = 0;
params.supervision_timeout = 200; // 2000ms (200 * 10ms)
params.rssi = -45; // Good signal strength

// Update device with connection parameters
let mac_addr = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC];
let mut device = BluetoothDeviceInfo::new(&mac_addr, b"Audio Device")?;
device.update_connection_params(&params);
```

#### Security Information

```rust
use renik::{BluetoothSecurityInfo, BluetoothDeviceInfo};

// Create security information
let mut security = BluetoothSecurityInfo::default();
security.link_key = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
                     0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10];
security.link_key_valid = 1;
security.authenticated = 1;
security.encrypted = 1;
security.security_level = 4; // High security

// Update device with security information
let mac_addr = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC];
let mut device = BluetoothDeviceInfo::new(&mac_addr, b"Secure Device")?;
device.update_security_info(&security);
```

### Persistent Storage

All configuration structures can be serialized for persistent storage:

```rust
use renik::{BluetoothDeviceInfo, BluetoothDeviceList};
use std::fs::File;
use std::io::{Read, Write};

// Create and configure device list
let mut device_list = BluetoothDeviceList::default();

let mac_address = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
let device = BluetoothDeviceInfo::new(&mac_address, b"Speaker")?;
device_list.add_device(device)?;

// Serialize to bytes
let list_bytes = unsafe {
    core::slice::from_raw_parts(
        &device_list as *const _ as *const u8,
        core::mem::size_of::<BluetoothDeviceList>()
    )
};

// Save to persistent storage
let mut file = File::create("bluetooth_devices.bin")?;
file.write_all(list_bytes)?;

// Later: Load from storage
let mut file = File::open("bluetooth_devices.bin")?;
let mut loaded_bytes = vec![0u8; core::mem::size_of::<BluetoothDeviceList>()];
file.read_exact(&mut loaded_bytes)?;

let loaded_list: BluetoothDeviceList = unsafe {
    core::ptr::read(loaded_bytes.as_ptr() as *const BluetoothDeviceList)
};

// Use loaded device list for reconnection
for i in 0..loaded_list.len() {
    let device = loaded_list.get_device(i)?;
    if device.is_paired() && device.supports_auto_reconnect() {
        println!("Attempting to reconnect to: {:?}", device.get_device_name());
        // Initiate reconnection...
    }
}
```

## Structure Sizes

All structures are optimized for embedded use with predictable memory footprints:

- `WifiConfig`: 104 bytes (32B SSID + 64B password + metadata)
- `BluetoothDeviceInfo`: ~200 bytes (includes connection params and security info)
- `BluetoothDeviceList`: ~2KB (10 devices + metadata)
- `BluetoothConnectionState`: ~220 bytes (device info + FSM state)
- `BluetoothConnectionParams`: 32 bytes (connection timing and quality metrics)
- `BluetoothSecurityInfo`: 32 bytes (authentication and encryption data)
- `ConnHandle`: 2 bytes (type-safe u16 wrapper with validation)
- `BluetoothConnectionPhase`: 1 byte (enum with u8 representation)
- `DeviceInfo`: 164 bytes (32B hardware ID + 128B secret + metadata)

## Bluetooth Device Types

`BluetoothDeviceInfo` automatically categorizes devices based on Class of Device:

- `DEVICE_TYPE_COMPUTER`: Desktop/laptop computers
- `DEVICE_TYPE_PHONE`: Mobile phones and smartphones  
- `DEVICE_TYPE_AUDIO`: Headphones, speakers, audio devices
- `DEVICE_TYPE_PERIPHERAL`: Keyboards, mice, input devices
- `DEVICE_TYPE_IMAGING`: Cameras, printers, scanners
- `DEVICE_TYPE_WEARABLE`: Smartwatches, fitness trackers
- `DEVICE_TYPE_TOY`: Gaming devices, toys
- `DEVICE_TYPE_NETWORK`: Network access points
- `DEVICE_TYPE_UNKNOWN`: Unrecognized or uncategorized devices

## Bluetooth Device Flags

`BluetoothDeviceInfo` supports comprehensive device capability and status flags:

- `FLAG_PAIRED`: Device is paired and authenticated
- `FLAG_TRUSTED`: Device is trusted for automatic connections
- `FLAG_AUDIO`: Device supports audio profiles (A2DP, HFP, etc.)
- `FLAG_INPUT`: Device supports input (HID profile - keyboards, mice)
- `FLAG_FILE_TRANSFER`: Device supports file transfer (OBEX, FTP)
- `FLAG_CONNECTED`: Device is currently connected
- `FLAG_AUTO_RECONNECT`: Device supports automatic reconnection
- `FLAG_RECENTLY_DISCOVERED`: Device was discovered in recent scan

## Error Handling

The library provides comprehensive error handling with specific error types:

- `Error::CredentialLengthExceeded`: Wi-Fi SSID (>32 bytes) or password (>64 bytes) too long
- `Error::IdentityLengthExceeded`: Device hardware ID (>32 bytes) or secret (>128 bytes) too long  
- `Error::InvalidBluetoothDeviceInfo`: Bluetooth device name (>32 bytes) or pairing key (>64 bytes) too long
- `Error::DeviceListFull`: Bluetooth device list already contains maximum devices (10)
- `Error::IndexOutOfBounds`: Attempted to access device at invalid index

All functions return `Result<T, Error>` for proper error handling:

```rust
use renik::{BluetoothDeviceInfo, Error};

match BluetoothDeviceInfo::new(&mac_addr, b"Very long device name that exceeds 32 bytes limit") {
    Ok(device) => println!("Device created successfully"),
    Err(Error::InvalidBluetoothDeviceInfo) => println!("Device name too long"),
    Err(e) => println!("Other error: {:?}", e),
}
```

## Use Cases

**Embedded Bluetooth Audio Systems**: Store paired speaker/headphone configurations with automatic reconnection capabilities.

**IoT Device Management**: Maintain device identity and Wi-Fi credentials across power cycles.

**Wearable Devices**: Manage connections to multiple peripherals (phones, sensors, accessories) with efficient storage.

**Industrial Automation**: Persistent device pairing for sensors, actuators, and control systems.

**Home Automation**: Store and manage connections to various smart home devices.

## License

The MIT License (MIT)
Copyright © 2025 rttf.dev

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the “Software”), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
