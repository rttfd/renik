#![cfg_attr(not(test), no_std)]

//! # Embedded Device Configuration
//!
//! This crate provides comprehensive configuration structures for embedded devices,
//! specifically for Wi-Fi connectivity, Bluetooth device management, and device
//! identification in `no_std` environments.
//!
//! ## Features
//!
//! - **Wi-Fi Configuration**: Store and manage Wi-Fi network credentials with `WifiConfig`
//! - **Bluetooth Device Management**: Complete Bluetooth device lifecycle management:
//!   - `BluetoothDeviceInfo`: Individual device information with pairing data
//!   - `BluetoothDeviceList`: Manage multiple paired devices (up to 10)
//!   - `BluetoothConnectionState`: Track connection status and link quality
//!   - `BluetoothConnectionParams`: Low-level connection parameters
//!   - `BluetoothSecurityInfo`: Security and authentication information
//! - **Device Identity**: Store device identification and authentication data with `DeviceInfo`
//! - **Memory Safe**: All structures use fixed-size buffers with length tracking
//! - **Serializable**: `#[repr(C)]` layout for easy persistence and IPC
//! - **Embedded Ready**: Full `no_std` compatibility with minimal dependencies
//!
//! ## Quick Start
//!
//! ```rust
//! use renik::{BluetoothDeviceInfo, BluetoothDeviceList, WifiConfig};
//!
//! // Wi-Fi configuration
//! let wifi = WifiConfig::new(b"MyNetwork", b"password123")?;
//!
//! // Bluetooth device management
//! let mac_addr = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC];
//! let device = BluetoothDeviceInfo::new(&mac_addr, b"My Speaker")?;
//!
//! let mut device_list = BluetoothDeviceList::default();
//! device_list.add_device(device)?;
//! # Ok::<(), renik::Error>(())
//! ```
//!
//! All structures are designed for persistent storage and can be safely serialized
//! to flash memory or other storage mediums for configuration persistence across
//! device reboots.

mod bluetooth;
mod device;
mod error;
mod wifi;

pub use bluetooth::{
    BluetoothConnectionParams, BluetoothConnectionPhase, BluetoothConnectionState,
    BluetoothDeviceInfo, BluetoothDeviceList, BluetoothSecurityInfo, ConnHandle,
};
pub use device::DeviceInfo;
pub use error::Error;
pub use wifi::WifiConfig;
