//! # Embedded Device Configuration
//!
//! This module provides configuration structures for embedded devices,
//! specifically for Wi-Fi connectivity and device identification in
//! no_std environments.

#![no_std]

use bytemuck::{Pod, Zeroable};

/// Magic number used to validate Wi-Fi configuration structures
/// Value: 0x57494649 (ASCII "WIFI")
const WIFI_CONFIG_MAGIC: u32 = 0x57494649;

/// Magic number used to validate device information structures  
/// Value: 0x444556 (ASCII "DEV")
const DEVICE_INFO_MAGIC: u32 = 0x444556;

/// Wi-Fi network configuration structure
///
/// This structure stores Wi-Fi credentials in a fixed-size format suitable
/// for embedded systems. It uses a magic number for validation and length
/// fields to track the actual size of variable-length data.
///
/// # Memory Layout
/// The structure uses `#[repr(C)]` to ensure predictable memory layout,
/// making it suitable for serialization and inter-process communication.
///
/// # Examples
/// ```
/// use renik::WifiConfig;
///
/// let mut config = WifiConfig::default();
/// config.set_credentials(b"MyNetwork", b"password123");
/// assert!(config.is_valid());
/// ```
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct WifiConfig {
    /// Magic number for structure validation (0x57494649)
    magic: u32, // 4-byte aligned
    /// Fixed-size buffer for network SSID (maximum 32 bytes)
    ssid: [u8; 32], // 1-byte aligned
    /// Fixed-size buffer for network password (maximum 64 bytes)
    password: [u8; 64], // 1-byte aligned
    /// Actual length of the SSID (0-32 bytes)
    ssid_len: u8, // 1-byte aligned
    /// Actual length of the password (0-64 bytes)
    password_len: u8, // 1-byte aligned
    /// Padding to align to a multiple of 4 if needed
    _padding: [u8; 2], // Ensures no implicit padding
}

impl Default for WifiConfig {
    /// Creates a new Wi-Fi configuration with default values
    ///
    /// The structure is initialized with the correct magic number
    /// and zero-length credentials.
    fn default() -> Self {
        Self {
            magic: WIFI_CONFIG_MAGIC,
            ssid_len: 0,
            password_len: 0,
            ssid: [0; 32],
            password: [0; 64],
            _padding: [0; 2],
        }
    }
}

impl WifiConfig {
    /// Validates the Wi-Fi configuration structure
    ///
    /// # Returns
    /// - `true` if the magic number is correct and SSID length is greater than 0
    /// - `false` otherwise
    ///
    /// # Note
    /// This method only checks for basic structural validity, not the
    /// actual correctness of the Wi-Fi credentials.
    pub fn is_valid(&self) -> bool {
        self.magic == WIFI_CONFIG_MAGIC && self.ssid_len > 0
    }

    /// Sets the Wi-Fi network credentials
    ///
    /// # Parameters
    /// - `ssid`: Network name as byte slice (max 32 bytes)
    /// - `password`: Network password as byte slice (max 64 bytes)
    ///
    /// # Returns
    /// - `true` if credentials were set successfully
    /// - `false` if either parameter exceeds maximum length
    ///
    /// # Behavior
    /// - Clears existing credential buffers before setting new values
    /// - Updates length fields to reflect actual credential sizes
    /// - Pads unused buffer space with zeros
    pub fn set_credentials(&mut self, ssid: &[u8], password: &[u8]) -> bool {
        if ssid.len() > 32 || password.len() > 64 {
            return false;
        }

        self.ssid_len = ssid.len() as u8;
        self.password_len = password.len() as u8;

        // Clear buffers to ensure no residual data
        self.ssid.fill(0);
        self.password.fill(0);

        // Copy new credentials into buffers
        self.ssid[..ssid.len()].copy_from_slice(ssid);
        self.password[..password.len()].copy_from_slice(password);

        true
    }

    /// Returns the stored SSID as a byte slice
    ///
    /// # Returns
    /// A slice containing only the valid SSID bytes (length determined by ssid_len)
    pub fn get_ssid(&self) -> &[u8] {
        &self.ssid[..self.ssid_len as usize]
    }

    /// Returns the stored password as a byte slice
    ///
    /// # Returns  
    /// A slice containing only the valid password bytes (length determined by password_len)
    pub fn get_password(&self) -> &[u8] {
        &self.password[..self.password_len as usize]
    }
}

/// Device identification and authentication structure
///
/// This structure stores device-specific information including a unique
/// hardware identifier and device secret. It's designed for
/// embedded systems that need to maintain device identity across reboots.
///
/// # Security Note
/// This structure stores sensitive authentication data. Ensure proper
/// memory protection and secure storage mechanisms when persisting this data.
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct DeviceInfo {
    /// Magic number for structure validation (0x444556)
    magic: u32, // 4-byte aligned
    /// Unique hardware identifier (16 bytes)
    hardware_id: [u8; 16], // 1-byte aligned
    /// Device secret (64 bytes)
    secret: [u8; 64], // 1-byte aligned
}

impl Default for DeviceInfo {
    /// Creates a new device info structure with default values
    ///
    /// The structure is initialized with the correct magic number
    /// and zeroed identifier/secret fields.
    fn default() -> Self {
        Self {
            magic: DEVICE_INFO_MAGIC,
            hardware_id: [0; 16],
            secret: [0; 64],
        }
    }
}

impl DeviceInfo {
    /// Validates the device information structure
    ///
    /// # Returns
    /// - `true` if the magic number is correct
    /// - `false` otherwise
    pub fn is_valid(&self) -> bool {
        self.magic == DEVICE_INFO_MAGIC
    }

    /// Sets the hardware identifier
    ///
    /// # Parameters
    /// - `hardware_id`: Hardware identifier as byte slice (max 16 bytes)
    ///
    /// # Returns
    /// - `true` if hardware ID was set successfully
    /// - `false` if the parameter exceeds 16 bytes
    ///
    /// # Note
    /// If the input is shorter than 16 bytes, only the specified bytes
    /// are updated, leaving the remainder unchanged.
    pub fn set_hardware_id(&mut self, hardware_id: &[u8]) -> bool {
        if hardware_id.len() > 16 {
            return false;
        }

        self.hardware_id[..hardware_id.len()].copy_from_slice(hardware_id);
        true
    }

    /// Sets the device secret
    ///
    /// # Parameters  
    /// - `secret`: Device secret as byte slice (max 64 bytes)
    ///
    /// # Returns
    /// - `true` if secret was set successfully
    /// - `false` if the parameter exceeds 64 bytes
    ///
    /// # Panics
    /// This method will panic if the slice conversion fails, which should
    /// not happen given the length check.
    pub fn set_secret(&mut self, secret: &[u8]) -> bool {
        if secret.len() > 64 {
            return false;
        }

        self.secret = secret[..secret.len()].try_into().unwrap();
        true
    }

    /// Returns the stored hardware identifier
    ///
    /// # Returns
    /// A reference to the complete 16-byte hardware identifier array
    pub fn get_hardware_id(&self) -> &[u8] {
        &self.hardware_id
    }

    /// Returns the stored device secret
    ///
    /// # Returns
    /// A reference to the complete 64-byte secret array
    pub fn get_secret(&self) -> &[u8] {
        &self.secret
    }
}
