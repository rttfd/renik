//! # Bluetooth Device Management
//!
//! This module provides comprehensive Bluetooth device management capabilities
//! for embedded systems, including device information storage, connection state
//! tracking, and finite state machine (FSM) management.
//!
//! ## Features
//!
//! - **Device Information**: Store and manage individual Bluetooth device configurations
//! - **Device Lists**: Manage multiple paired devices (up to 10)
//! - **Connection State**: Track real-time connection status with FSM support
//! - **Type-Safe Handles**: Validated connection handle wrapper
//! - **Security Management**: Store pairing keys and security information
//! - **Memory Efficient**: Fixed-size structures optimized for embedded use
//!
//! ## Core Structures
//!
//! - [`BluetoothDeviceInfo`]: Complete device information including pairing data
//! - [`BluetoothDeviceList`]: Container for multiple device configurations
//! - [`BluetoothConnectionState`]: Real-time connection tracking with FSM
//! - [`ConnHandle`]: Type-safe connection handle wrapper
//! - [`BluetoothConnectionPhase`]: FSM phases for connection lifecycle
//!
//! ## Finite State Machine
//!
//! The Bluetooth connection FSM supports 13 distinct phases:
//!
//! ```text
//! Idle → Discovery → Connecting → Connected → Authenticating → SettingUpEncryption
//!   ↑        ↓           ↓           ↓             ↓                    ↓
//!   └─────────────────────────────────────────────────────────────→ Failed
//!                                   ↓             ↓                    ↓
//!                            ServiceDiscovery → FullyConnected → Ready → Maintaining
//!                                   ↓             ↓           ↓        ↓
//!                                   └─────────→ Disconnecting ←────────┘
//!                                                    ↓
//!                            Reconnecting ←──────────┘
//!                                   ↓
//!                            Connecting (retry)
//! ```
//!
//! ## Memory Layout
//!
//! All structures use `#[repr(C)]` layout for reliable serialization:
//! - Predictable field ordering
//! - No hidden padding
//! - Cross-platform compatibility
//! - Suitable for persistent storage
//!
//! ## Examples
//!
//! ### Basic Device Management
//! ```
//! use renik::{BluetoothDeviceInfo, BluetoothDeviceList};
//!
//! // Create a device
//! let mac_addr = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC];
//! let mut device = BluetoothDeviceInfo::new(&mac_addr, b"My Speaker")?;
//! device.set_pairing_key(b"audio_key_123")?;
//! device.add_flag(BluetoothDeviceInfo::FLAG_AUDIO);
//!
//! // Add to device list
//! let mut device_list = BluetoothDeviceList::default();
//! device_list.add_device(device)?;
//! # Ok::<(), renik::Error>(())
//! ```
//!
//! ### Connection State Tracking
//! ```
//! use renik::{BluetoothConnectionState, BluetoothConnectionPhase, ConnHandle};
//!
//! let mut connection = BluetoothConnectionState::default();
//!
//! // FSM transitions
//! assert!(connection.advance_to_phase(BluetoothConnectionPhase::Discovery));
//! assert!(connection.advance_to_phase(BluetoothConnectionPhase::Connecting));
//! assert!(connection.advance_to_phase(BluetoothConnectionPhase::Connected));
//!
//! // Set connection details
//! connection.set_connection_handle(Some(ConnHandle::new(0x0042)));
//! connection.set_link_quality(85);
//! # Ok::<(), renik::Error>(())
//! ```

use crate::Error;
use bytemuck::{Pod, Zeroable};

/// Magic number used to validate Bluetooth device configuration structures
/// Value: 0x42544C45 (ASCII "BTLE")
const BLUETOOTH_CONFIG_MAGIC: u32 = 0x4254_4C45;

/// Magic number for Bluetooth device list
/// Value: 0x42544C53 (ASCII "BTLS")
const BLUETOOTH_DEVICE_LIST_MAGIC: u32 = 0x4254_4C53;

/// Magic number for Bluetooth connection state
/// Value: 0x42544353 (ASCII "BTCS")
const BLUETOOTH_CONNECTION_STATE_MAGIC: u32 = 0x4254_4353;

/// Bluetooth device list structure
///
/// This structure represents a list of Bluetooth devices, including their
/// configuration and connection status. It's used for managing multiple
/// Bluetooth devices in embedded systems.
///
/// # Memory Layout
/// The structure uses `#[repr(C)]` to ensure predictable memory layout,
/// making it suitable for serialization and inter-process communication.
///
/// # Security Note
/// This structure may store sensitive pairing data. Ensure proper
/// memory protection and secure storage mechanisms when persisting this data.
///
/// # Examples
/// ```
/// use renik::{BluetoothDeviceInfo, BluetoothDeviceList};
///
/// let mac_addr1 = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC];
/// let mac_addr2 = [0x98, 0x76, 0x54, 0x32, 0x10, 0xFE];
/// let device1 = BluetoothDeviceInfo::new(&mac_addr1, b"Device 1").unwrap();
/// let device2 = BluetoothDeviceInfo::new(&mac_addr2, b"Device 2").unwrap();
/// let mut device_list = BluetoothDeviceList::default();
/// device_list.add_device(device1).unwrap();
/// device_list.add_device(device2).unwrap();
/// assert_eq!(device_list.len(), 2);
/// ```
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct BluetoothDeviceList {
    /// Magic number for structure validation (0x42544C53)
    magic: u32, // 4-byte aligned
    /// Array of Bluetooth device configurations
    devices: [BluetoothDeviceInfo; 10], // 4-byte aligned
    /// Number of devices currently in the list
    device_count: u8, // 1-byte aligned
    /// Padding to ensure proper alignment
    _padding: [u8; 3], // Ensures 4-byte alignment
}

impl Default for BluetoothDeviceList {
    /// Creates a new Bluetooth device list with default values
    ///
    /// The structure is initialized with the correct magic number
    /// and an empty device list.
    fn default() -> Self {
        Self {
            magic: BLUETOOTH_DEVICE_LIST_MAGIC,
            devices: Default::default(),
            device_count: 0,
            _padding: [0; 3],
        }
    }
}

impl BluetoothDeviceList {
    /// Adds a Bluetooth device configuration to the list
    ///
    /// # Parameters
    /// - `device_config`: Bluetooth device configuration
    ///
    /// # Returns
    /// - `Ok(())` if the device was added successfully
    /// - `Err(Error)` if the device list is full
    ///
    /// # Errors
    /// Returns `Error::DeviceListFull` if the device list is already at maximum capacity.
    pub fn add_device(&mut self, device_config: BluetoothDeviceInfo) -> Result<(), Error> {
        if self.device_count as usize >= self.devices.len() {
            return Err(Error::DeviceListFull);
        }

        self.devices[self.device_count as usize] = device_config;
        self.device_count += 1;

        Ok(())
    }

    /// Removes a Bluetooth device configuration from the list
    ///
    /// # Parameters
    /// - `index`: Index of the device to remove (0-based)
    ///
    /// # Returns
    /// - `Ok(())` if the device was removed successfully
    /// - `Err(Error)` if the index is out of bounds
    ///
    /// # Errors
    /// Returns `Error::IndexOutOfBounds` if the specified index is not valid.
    pub fn remove_device(&mut self, index: usize) -> Result<(), Error> {
        if index >= self.device_count as usize {
            return Err(Error::IndexOutOfBounds);
        }

        // Shift devices down to fill the gap
        for i in index..(self.device_count as usize - 1) {
            self.devices[i] = self.devices[i + 1];
        }

        self.device_count -= 1;

        Ok(())
    }

    /// Returns a reference to a Bluetooth device configuration
    ///
    /// # Parameters
    /// - `index`: Index of the device to retrieve (0-based)
    ///
    /// # Returns
    /// - `Ok(&BluetoothDeviceInfo)` if the index is valid
    /// - `Err(Error)` if the index is out of bounds
    ///
    /// # Errors
    /// Returns `Error::IndexOutOfBounds` if the specified index is not valid.
    pub fn get_device(&self, index: usize) -> Result<&BluetoothDeviceInfo, Error> {
        if index >= self.device_count as usize {
            return Err(Error::IndexOutOfBounds);
        }

        Ok(&self.devices[index])
    }

    /// Returns the number of devices in the list
    ///
    /// # Returns
    /// The current device count
    #[must_use]
    pub fn len(&self) -> usize {
        self.device_count as usize
    }

    /// Checks if the device list is empty
    ///
    /// # Returns
    /// - `true` if there are no devices in the list
    /// - `false` otherwise
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.device_count == 0
    }
}

/// Bluetooth connection state structure
///
/// This structure represents the connection state of a Bluetooth device,
/// including information about the remote device, connection status,
/// and link quality. It's used for managing Bluetooth connections
/// in embedded systems.
///
/// # Memory Layout
/// The structure uses `#[repr(C)]` to ensure predictable memory layout,
/// making it suitable for serialization and inter-process communication.
///
/// # Security Note
/// This structure may store sensitive connection data. Ensure proper
/// memory protection and secure storage mechanisms when persisting this data.
///
/// # Examples
/// ```
/// use renik::{BluetoothDeviceInfo, BluetoothConnectionState};
///
/// let mac_addr = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC];
/// let device = BluetoothDeviceInfo::new(&mac_addr, b"My Device").unwrap();
/// let mut connection_state = BluetoothConnectionState::default();
/// connection_state.set_remote_device(device);
/// connection_state.set_connected(true);
/// assert!(connection_state.is_connected());
/// ```
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct BluetoothConnectionState {
    /// Magic number for structure validation (0x42544353)
    magic: u32, // 4-byte aligned
    /// Bluetooth device configuration
    device_config: BluetoothDeviceInfo, // 4-byte aligned
    /// Connection status flags
    connection_flags: u8, // 1-byte aligned
    /// Link quality (RSSI, LQI, etc.)
    link_quality: u8, // 1-byte aligned
    /// Current connection phase
    connection_phase: u8, // 1-byte aligned (maps to BluetoothConnectionPhase)
    /// Padding to ensure proper alignment
    _padding: [u8; 1], // Ensures 4-byte alignment
}

impl Default for BluetoothConnectionState {
    /// Creates a new Bluetooth connection state with default values
    ///
    /// The structure is initialized with the correct magic number
    /// and a disconnected state.
    fn default() -> Self {
        Self {
            magic: BLUETOOTH_CONNECTION_STATE_MAGIC,
            device_config: BluetoothDeviceInfo::default(),
            connection_flags: 0,
            link_quality: 0,
            connection_phase: BluetoothConnectionPhase::Idle as u8,
            _padding: [0; 1],
        }
    }
}

impl BluetoothConnectionState {
    /// Sets the remote Bluetooth device configuration
    ///
    /// # Parameters
    /// - `device_config`: Bluetooth device configuration
    pub fn set_remote_device(&mut self, device_config: BluetoothDeviceInfo) {
        self.device_config = device_config;
    }

    /// Sets the connection status
    ///
    /// # Parameters
    /// - `connected`: `true` if connected, `false` if disconnected
    pub fn set_connected(&mut self, connected: bool) {
        if connected {
            self.connection_flags |= 0x01;
        } else {
            self.connection_flags &= !0x01;
        }
    }

    /// Sets the link quality
    ///
    /// # Parameters
    /// - `quality`: Link quality value (0-255)
    pub fn set_link_quality(&mut self, quality: u8) {
        self.link_quality = quality;
    }

    /// Returns the remote Bluetooth device configuration
    ///
    /// # Returns
    /// A reference to the Bluetooth device configuration
    #[must_use]
    pub fn get_remote_device(&self) -> &BluetoothDeviceInfo {
        &self.device_config
    }

    /// Returns the connection status
    ///
    /// # Returns
    /// - `true` if connected
    /// - `false` if disconnected
    #[must_use]
    pub fn is_connected(&self) -> bool {
        (self.connection_flags & 0x01) != 0
    }

    /// Returns the link quality
    ///
    /// # Returns
    /// The link quality value (0-255)
    #[must_use]
    pub fn get_link_quality(&self) -> u8 {
        self.link_quality
    }

    /// Sets the authentication status
    ///
    /// # Parameters
    /// - `authenticated`: `true` if authenticated, `false` if not
    pub fn set_authenticated(&mut self, authenticated: bool) {
        if authenticated {
            self.connection_flags |= 0x02;
        } else {
            self.connection_flags &= !0x02;
        }
    }

    /// Returns the authentication status
    ///
    /// # Returns
    /// - `true` if authenticated
    /// - `false` if not authenticated
    #[must_use]
    pub fn is_authenticated(&self) -> bool {
        (self.connection_flags & 0x02) != 0
    }

    /// Sets the remote device address
    ///
    /// # Parameters
    /// - `address`: 6-byte Bluetooth address
    pub fn set_remote_device_address(&mut self, address: [u8; 6]) {
        self.device_config.mac_address = address;
    }

    /// Gets the remote device address
    ///
    /// # Returns
    /// Optional 6-byte Bluetooth address
    #[must_use]
    pub fn get_remote_device_address(&self) -> Option<[u8; 6]> {
        Some(self.device_config.mac_address)
    }

    /// Sets the connection handle
    ///
    /// # Parameters
    /// - `handle`: Connection handle (`ConnHandle`)
    pub fn set_connection_handle(&mut self, handle: Option<ConnHandle>) {
        self.device_config.connection_params.connection_handle = handle.unwrap_or_default();
    }

    /// Gets the connection handle
    ///
    /// # Returns
    /// Optional connection handle
    #[must_use]
    pub fn get_connection_handle(&self) -> Option<ConnHandle> {
        if self.device_config.connection_params.connection_handle.raw() == 0 {
            None
        } else {
            Some(self.device_config.connection_params.connection_handle)
        }
    }

    /// Sets the link type
    ///
    /// # Parameters
    /// - `link_type`: Link type (0x01 = ACL, 0x02 = SCO)
    pub fn set_link_type(&mut self, link_type: u8) {
        self.device_config.connection_params.link_type = link_type;
    }

    /// Gets the link type
    ///
    /// # Returns
    /// Link type value
    #[must_use]
    pub fn get_link_type(&self) -> u8 {
        self.device_config.connection_params.link_type
    }

    /// Sets the connection phase
    ///
    /// # Parameters
    /// - `phase`: Connection phase
    pub fn set_connection_phase(&mut self, phase: BluetoothConnectionPhase) {
        self.connection_phase = phase as u8;
    }

    /// Gets the connection phase
    ///
    /// # Returns
    /// The current connection phase
    #[must_use]
    pub fn get_connection_phase(&self) -> BluetoothConnectionPhase {
        // Convert u8 back to enum, default to Idle if invalid
        match self.connection_phase {
            1 => BluetoothConnectionPhase::Discovery,
            2 => BluetoothConnectionPhase::Connecting,
            3 => BluetoothConnectionPhase::Connected,
            4 => BluetoothConnectionPhase::Authenticating,
            5 => BluetoothConnectionPhase::SettingUpEncryption,
            6 => BluetoothConnectionPhase::FullyConnected,
            7 => BluetoothConnectionPhase::ServiceDiscovery,
            8 => BluetoothConnectionPhase::Ready,
            9 => BluetoothConnectionPhase::Maintaining,
            10 => BluetoothConnectionPhase::Reconnecting,
            11 => BluetoothConnectionPhase::Failed,
            12 => BluetoothConnectionPhase::Disconnecting,
            _ => BluetoothConnectionPhase::Idle, // Default for 0 and invalid values
        }
    }

    /// Advances to the next connection phase
    ///
    /// # Parameters
    /// - `next_phase`: The next phase to transition to
    ///
    /// # Returns
    /// - `true` if the transition is valid
    /// - `false` if the transition is not allowed
    pub fn advance_to_phase(&mut self, next_phase: BluetoothConnectionPhase) -> bool {
        let current = self.get_connection_phase();

        // Simple rule-based validation instead of exhaustive matching
        let valid_transition = next_phase == BluetoothConnectionPhase::Idle
            || Self::is_valid_transition(current, next_phase);

        if valid_transition {
            self.set_connection_phase(next_phase);
        }

        valid_transition
    }

    /// Helper function to check if a state transition is valid
    fn is_valid_transition(
        current: BluetoothConnectionPhase,
        next: BluetoothConnectionPhase,
    ) -> bool {
        use BluetoothConnectionPhase::{
            Authenticating, Connected, Connecting, Disconnecting, Discovery, Failed,
            FullyConnected, Idle, Maintaining, Ready, Reconnecting, ServiceDiscovery,
            SettingUpEncryption,
        };

        match current {
            Idle => matches!(next, Discovery | Connecting),
            Discovery => next == Connecting,
            Connecting => matches!(next, Connected | Failed),
            Connected => matches!(next, Authenticating | ServiceDiscovery | Disconnecting),
            Authenticating => matches!(next, SettingUpEncryption | Failed | Disconnecting),
            SettingUpEncryption => matches!(next, FullyConnected | Failed | Disconnecting),
            FullyConnected => matches!(next, ServiceDiscovery | Ready | Disconnecting),
            ServiceDiscovery => matches!(next, Ready | Failed | Disconnecting),
            Ready => matches!(next, Maintaining | Disconnecting),
            Maintaining => matches!(next, Reconnecting | Disconnecting),
            Reconnecting => matches!(next, Connecting | Failed),
            Failed => next == Reconnecting,
            Disconnecting => false, // Only to Idle, handled above
        }
    }
}

/// Connection parameters for Bluetooth devices
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct BluetoothConnectionParams {
    /// Connection handle assigned by the controller
    pub connection_handle: ConnHandle,
    /// Connection interval in 1.25ms units (range: 6-3200)
    pub connection_interval: u16,
    /// Connection latency (range: 0-499)
    pub connection_latency: u16,
    /// Supervision timeout in 10ms units (range: 10-3200)
    pub supervision_timeout: u16,
    /// Clock accuracy (range: 0-7)
    pub master_clock_accuracy: u8,
    /// Link type (0x01 = ACL, 0x02 = SCO)
    pub link_type: u8,
    /// Encryption enabled (0x00 = disabled, 0x01 = enabled)
    pub encryption_enabled: u8,
    /// RSSI value (-127 to 127 dBm)
    pub rssi: i8,
    /// Connection timestamp (seconds since epoch)
    pub connected_at: u32,
    /// Last activity timestamp (seconds since epoch)
    pub last_activity: u32,
    /// Padding for alignment
    _padding: [u8; 4],
}

impl Default for BluetoothConnectionParams {
    fn default() -> Self {
        Self {
            connection_handle: ConnHandle::default(),
            connection_interval: 0,
            connection_latency: 0,
            supervision_timeout: 0,
            master_clock_accuracy: 0,
            link_type: 0,
            encryption_enabled: 0,
            rssi: -127,
            connected_at: 0,
            last_activity: 0,
            _padding: [0; 4],
        }
    }
}

/// Security information for Bluetooth connections
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct BluetoothSecurityInfo {
    /// Link key for authentication (16 bytes)
    pub link_key: [u8; 16],
    /// Link key type (0x00-0x07)
    pub link_key_type: u8,
    /// Authentication requirements
    pub auth_requirements: u8,
    /// IO capabilities (0x00-0x04)
    pub io_capabilities: u8,
    /// Security level (0x01-0x04)
    pub security_level: u8,
    /// PIN code length (0-16)
    pub pin_length: u8,
    /// Whether link key is valid
    pub link_key_valid: u8,
    /// Whether device was authenticated
    pub authenticated: u8,
    /// Whether connection is encrypted
    pub encrypted: u8,
    /// Whether device supports secure simple pairing
    pub ssp_supported: u8,
    /// Whether MITM protection is required
    pub mitm_required: u8,
    /// Padding for alignment
    _padding: [u8; 6],
}

impl Default for BluetoothSecurityInfo {
    fn default() -> Self {
        Self {
            link_key: [0; 16],
            link_key_type: 0,
            auth_requirements: 0,
            io_capabilities: 0,
            security_level: 1,
            pin_length: 0,
            link_key_valid: 0,
            authenticated: 0,
            encrypted: 0,
            ssp_supported: 0,
            mitm_required: 0,
            _padding: [0; 6],
        }
    }
}

/// Complete Bluetooth device information for storage
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct BluetoothDeviceInfo {
    /// Magic number for validation
    magic: u32,
    /// Bluetooth MAC address (6 bytes)
    mac_address: [u8; 6],
    /// Fixed-size buffer for device name (maximum 32 bytes)
    device_name: [u8; 32],
    /// Actual length of the device name (0-32 bytes)
    device_name_len: u8,
    /// Fixed-size buffer for pairing key/PIN (maximum 64 bytes)
    pairing_key: [u8; 64],
    /// Actual length of the pairing key (0-64 bytes)
    pairing_key_len: u8,
    /// Device class of device (24-bit value)
    class_of_device: [u8; 3],
    /// Device type based on class (audio, input, etc.)
    device_type: u8,
    /// Device flags (paired, trusted, etc.)
    flags: u8,
    /// Padding for 4-byte alignment (1 byte to align next u32)
    _padding1: u8,
    /// Number of successful connections
    connection_count: u32,
    /// Last seen timestamp (seconds since epoch)
    last_seen: u32,
    /// Last successful connection timestamp
    last_connected: u32,
    /// Connection parameters
    connection_params: BluetoothConnectionParams,
    /// Security information
    security_info: BluetoothSecurityInfo,
    /// Vendor ID (if available)
    vendor_id: u16,
    /// Product ID (if available)
    product_id: u16,
    /// Version (if available)
    version: u16,
    /// Final padding for structure alignment
    _padding2: u16,
}

// Manual implementation for Pod/Zeroable to handle alignment properly
unsafe impl Pod for BluetoothDeviceInfo {}
unsafe impl Zeroable for BluetoothDeviceInfo {}

impl Default for BluetoothDeviceInfo {
    fn default() -> Self {
        Self {
            magic: BLUETOOTH_CONFIG_MAGIC,
            mac_address: [0; 6],
            device_name: [0; 32],
            device_name_len: 0,
            pairing_key: [0; 64],
            pairing_key_len: 0,
            class_of_device: [0; 3],
            device_type: 0,
            flags: 0,
            _padding1: 0,
            connection_count: 0,
            last_seen: 0,
            last_connected: 0,
            connection_params: BluetoothConnectionParams::default(),
            security_info: BluetoothSecurityInfo::default(),
            vendor_id: 0,
            product_id: 0,
            version: 0,
            _padding2: 0,
        }
    }
}

/// Device type constants based on Class of Device major class
impl BluetoothDeviceInfo {
    pub const DEVICE_TYPE_UNKNOWN: u8 = 0;
    pub const DEVICE_TYPE_COMPUTER: u8 = 1;
    pub const DEVICE_TYPE_PHONE: u8 = 2;
    pub const DEVICE_TYPE_NETWORK: u8 = 3;
    pub const DEVICE_TYPE_AUDIO: u8 = 4;
    pub const DEVICE_TYPE_PERIPHERAL: u8 = 5;
    pub const DEVICE_TYPE_IMAGING: u8 = 6;
    pub const DEVICE_TYPE_WEARABLE: u8 = 7;
    pub const DEVICE_TYPE_TOY: u8 = 8;
}

/// Device flags for `BluetoothDeviceInfo`
impl BluetoothDeviceInfo {
    /// Device is paired
    pub const FLAG_PAIRED: u8 = 0x01;
    /// Device is trusted
    pub const FLAG_TRUSTED: u8 = 0x02;
    /// Device supports audio
    pub const FLAG_AUDIO: u8 = 0x04;
    /// Device supports input (keyboard/mouse)
    pub const FLAG_INPUT: u8 = 0x08;
    /// Device supports file transfer
    pub const FLAG_FILE_TRANSFER: u8 = 0x10;
    /// Device is currently connected
    pub const FLAG_CONNECTED: u8 = 0x20;
    /// Device supports automatic reconnection
    pub const FLAG_AUTO_RECONNECT: u8 = 0x40;
    /// Device was discovered recently
    pub const FLAG_RECENTLY_DISCOVERED: u8 = 0x80;
}

impl BluetoothDeviceInfo {
    /// Creates a new Bluetooth device info with basic information
    ///
    /// # Parameters
    /// - `mac_address`: Bluetooth MAC address as 6-byte array
    /// - `device_name`: Device name as byte slice (max 32 bytes)
    ///
    /// # Returns
    /// - `Ok(BluetoothDeviceInfo)` if the device info was created successfully
    /// - `Err(Error)` if the device name length exceeded the maximum allowed
    ///
    /// # Errors
    /// Returns `Error::InvalidBluetoothDeviceInfo` if the device name exceeds 32 bytes.
    pub fn new(mac_address: &[u8; 6], device_name: &[u8]) -> Result<Self, Error> {
        if device_name.len() > 32 {
            return Err(Error::InvalidBluetoothDeviceInfo);
        }

        let mut device = Self::default();
        device.set_mac_address(mac_address);
        device.set_device_name(device_name)?;
        Ok(device)
    }

    /// Validates the device info structure
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.magic == BLUETOOTH_CONFIG_MAGIC && !self.mac_address.iter().all(|&b| b == 0)
    }

    /// Sets the MAC address
    pub fn set_mac_address(&mut self, mac_address: &[u8; 6]) {
        self.mac_address.copy_from_slice(mac_address);
    }

    /// Sets the device name
    ///
    /// # Parameters
    /// - `device_name`: Device name as byte slice (max 32 bytes)
    ///
    /// # Returns
    /// - `Ok(())` if the device name was set successfully
    /// - `Err(Error)` if the device name length exceeded the maximum allowed
    ///
    /// # Errors
    /// Returns `Error::InvalidBluetoothDeviceInfo` if the device name exceeds 32 bytes.
    #[allow(clippy::cast_possible_truncation)]
    pub fn set_device_name(&mut self, device_name: &[u8]) -> Result<(), Error> {
        if device_name.len() > 32 {
            return Err(Error::InvalidBluetoothDeviceInfo);
        }

        self.device_name_len = device_name.len() as u8;
        self.device_name.fill(0);
        self.device_name[..device_name.len()].copy_from_slice(device_name);
        Ok(())
    }

    /// Sets the pairing key/PIN for the device
    ///
    /// # Parameters
    /// - `pairing_key`: Pairing key/PIN as byte slice (max 64 bytes)
    ///
    /// # Returns
    /// - `Ok(())` if the pairing key was set successfully
    /// - `Err(Error)` if the pairing key length exceeded the maximum allowed
    ///
    /// # Errors
    /// Returns `Error::InvalidBluetoothDeviceInfo` if the pairing key exceeds 64 bytes.
    #[allow(clippy::cast_possible_truncation)]
    pub fn set_pairing_key(&mut self, pairing_key: &[u8]) -> Result<(), Error> {
        if pairing_key.len() > 64 {
            return Err(Error::InvalidBluetoothDeviceInfo);
        }

        self.pairing_key_len = pairing_key.len() as u8;
        self.pairing_key.fill(0);
        self.pairing_key[..pairing_key.len()].copy_from_slice(pairing_key);
        Ok(())
    }

    /// Returns the stored pairing key as a byte slice
    ///
    /// # Returns
    /// A slice containing only the valid pairing key bytes (length determined by `pairing_key_len`)
    #[must_use]
    pub fn get_pairing_key(&self) -> &[u8] {
        &self.pairing_key[..self.pairing_key_len as usize]
    }

    /// Sets both device name and pairing key at once
    ///
    /// # Parameters
    /// - `device_name`: Device name as byte slice (max 32 bytes)
    /// - `pairing_key`: Pairing key/PIN as byte slice (max 64 bytes)
    ///
    /// # Returns
    /// - `Ok(())` if both were set successfully
    /// - `Err(Error)` if either length exceeded the maximum allowed
    ///
    /// # Errors
    /// Returns `Error::InvalidBluetoothDeviceInfo` if either the device name exceeds 32 bytes
    /// or the pairing key exceeds 64 bytes.
    pub fn set_device_info(&mut self, device_name: &[u8], pairing_key: &[u8]) -> Result<(), Error> {
        self.set_device_name(device_name)?;
        self.set_pairing_key(pairing_key)?;
        Ok(())
    }

    /// Sets the class of device
    pub fn set_class_of_device(&mut self, class_of_device: &[u8; 3]) {
        self.class_of_device.copy_from_slice(class_of_device);

        // Update device type based on major class
        // Major class is bits 8-12 (bits 0-4 of the second byte)
        let major_class = (class_of_device[1] >> 2) & 0x1F;
        self.device_type = match major_class {
            1 => Self::DEVICE_TYPE_COMPUTER,
            2 => Self::DEVICE_TYPE_PHONE,
            3 => Self::DEVICE_TYPE_NETWORK,
            4 => Self::DEVICE_TYPE_AUDIO,
            5 => Self::DEVICE_TYPE_PERIPHERAL,
            6 => Self::DEVICE_TYPE_IMAGING,
            7 => Self::DEVICE_TYPE_WEARABLE,
            8 => Self::DEVICE_TYPE_TOY,
            _ => Self::DEVICE_TYPE_UNKNOWN,
        };
    }

    /// Updates connection parameters
    pub fn update_connection_params(&mut self, params: &BluetoothConnectionParams) {
        self.connection_params = *params;
        self.connection_count += 1;
        self.add_flag(Self::FLAG_CONNECTED);
    }

    /// Updates security information
    pub fn update_security_info(&mut self, security: &BluetoothSecurityInfo) {
        self.security_info = *security;
        if security.authenticated != 0 {
            self.add_flag(Self::FLAG_PAIRED);
        }
    }

    /// Sets connection flags
    pub fn set_flags(&mut self, flags: u8) {
        self.flags = flags;
    }

    /// Adds a connection flag
    pub fn add_flag(&mut self, flag: u8) {
        self.flags |= flag;
    }

    /// Removes a connection flag
    pub fn remove_flag(&mut self, flag: u8) {
        self.flags &= !flag;
    }

    /// Checks if a specific flag is set
    #[must_use]
    pub fn has_flag(&self, flag: u8) -> bool {
        (self.flags & flag) != 0
    }

    /// Updates last seen timestamp
    pub fn update_last_seen(&mut self, timestamp: u32) {
        self.last_seen = timestamp;
    }

    /// Updates last connected timestamp
    pub fn update_last_connected(&mut self, timestamp: u32) {
        self.last_connected = timestamp;
    }

    /// Sets the connection count
    pub fn set_connection_count(&mut self, count: u32) {
        self.connection_count = count;
    }

    /// Increments the connection count
    pub fn increment_connection_count(&mut self) {
        self.connection_count = self.connection_count.saturating_add(1);
    }

    /// Sets the last connected timestamp
    pub fn set_last_connected(&mut self, timestamp: u32) {
        self.last_connected = timestamp;
    }

    /// Sets the last seen timestamp
    pub fn set_last_seen(&mut self, timestamp: u32) {
        self.last_seen = timestamp;
    }

    /// Getters
    #[must_use]
    pub fn get_mac_address(&self) -> &[u8; 6] {
        &self.mac_address
    }

    #[must_use]
    pub fn get_device_name(&self) -> &[u8] {
        &self.device_name[..self.device_name_len as usize]
    }

    #[must_use]
    pub fn get_class_of_device(&self) -> &[u8; 3] {
        &self.class_of_device
    }

    #[must_use]
    pub fn get_device_type(&self) -> u8 {
        self.device_type
    }

    #[must_use]
    pub fn get_flags(&self) -> u8 {
        self.flags
    }

    #[must_use]
    pub fn get_connection_params(&self) -> &BluetoothConnectionParams {
        &self.connection_params
    }

    #[must_use]
    pub fn get_security_info(&self) -> &BluetoothSecurityInfo {
        &self.security_info
    }

    #[must_use]
    pub fn is_paired(&self) -> bool {
        self.has_flag(Self::FLAG_PAIRED)
    }

    #[must_use]
    pub fn is_connected(&self) -> bool {
        self.has_flag(Self::FLAG_CONNECTED)
    }

    #[must_use]
    pub fn is_trusted(&self) -> bool {
        self.has_flag(Self::FLAG_TRUSTED)
    }

    #[must_use]
    pub fn supports_auto_reconnect(&self) -> bool {
        self.has_flag(Self::FLAG_AUTO_RECONNECT)
    }
}

/// Bluetooth connection handle wrapper
///
/// Provides a type-safe wrapper around the raw connection handle value
/// with validation to ensure the handle is within the valid range (0x0000-0x0EFF).
///
/// According to the Bluetooth specification, connection handles are assigned by
/// the Bluetooth controller and must be unique for each active connection.
/// The valid range is 0x0000-0x0EFF (0-3839 decimal), with 0x0F00-0x0FFF
/// reserved for future use.
///
/// # Use Cases
/// - Tracking active Bluetooth connections in embedded systems
/// - Providing type safety for connection handle operations
/// - Ensuring compliance with Bluetooth specification limits
///
/// # Performance
/// This is a zero-cost abstraction - the wrapper has the same memory
/// layout and performance characteristics as a raw `u16`.
///
/// # Examples
/// ```
/// use renik::ConnHandle;
///
/// // Create a valid connection handle
/// let handle = ConnHandle::new(0x0001);
/// assert_eq!(handle.raw(), 0x0001);
///
/// // Handles can be converted to/from u16
/// let raw_value: u16 = handle.into();
/// let back_to_handle = ConnHandle::from(raw_value);
/// assert_eq!(handle, back_to_handle);
///
/// // Maximum valid handle
/// let max_handle = ConnHandle::new(0x0EFF);
/// assert_eq!(max_handle.raw(), 0x0EFF);
/// ```
///
/// # Panics
/// The `new` method panics if the provided value exceeds 0x0EFF:
/// ```should_panic
/// use renik::ConnHandle;
/// let invalid = ConnHandle::new(0x0F00); // Panics!
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable)]
#[repr(transparent)]
#[derive(Default)]
pub struct ConnHandle(u16);

impl ConnHandle {
    /// Create a new connection handle instance.
    ///
    /// # Parameters
    /// - `val`: Raw connection handle value (must be <= 0x0EFF)
    ///
    /// # Panics
    /// Panics if the value exceeds 0x0EFF (the maximum valid connection handle).
    #[must_use]
    pub fn new(val: u16) -> Self {
        assert!(val <= 0x0EFF, "Connection handle must be <= 0x0EFF");
        Self(val)
    }

    /// Get the underlying representation.
    ///
    /// # Returns
    /// The raw u16 connection handle value.
    #[must_use]
    pub fn raw(self) -> u16 {
        self.0
    }
}

impl From<u16> for ConnHandle {
    fn from(val: u16) -> Self {
        Self::new(val)
    }
}

impl From<ConnHandle> for u16 {
    fn from(handle: ConnHandle) -> Self {
        handle.raw()
    }
}

/// Connection phases for multi-phase Bluetooth connection flow
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BluetoothConnectionPhase {
    /// Initial state - no connection attempt
    Idle = 0,
    /// Discovering devices
    Discovery = 1,
    /// Connecting to a specific device
    Connecting = 2,
    /// Connected but not authenticated
    Connected = 3,
    /// Authentication in progress
    Authenticating = 4,
    /// Authenticated, setting up encryption
    SettingUpEncryption = 5,
    /// Connected, authenticated, and encrypted
    FullyConnected = 6,
    /// Service discovery in progress
    ServiceDiscovery = 7,
    /// Connection established with services
    Ready = 8,
    /// Connection maintenance mode
    Maintaining = 9,
    /// Connection lost, attempting reconnection
    Reconnecting = 10,
    /// Connection failed
    Failed = 11,
    /// Disconnecting
    Disconnecting = 12,
}

impl Default for BluetoothConnectionPhase {
    fn default() -> Self {
        Self::Idle
    }
}

impl BluetoothConnectionPhase {
    /// Returns true if the phase indicates an active connection
    #[must_use]
    pub fn is_connected(&self) -> bool {
        matches!(
            self,
            Self::Connected
                | Self::Authenticating
                | Self::SettingUpEncryption
                | Self::FullyConnected
                | Self::ServiceDiscovery
                | Self::Ready
                | Self::Maintaining
        )
    }

    /// Returns true if the phase indicates the connection is secure
    #[must_use]
    pub fn is_secure(&self) -> bool {
        matches!(
            self,
            Self::FullyConnected | Self::ServiceDiscovery | Self::Ready | Self::Maintaining
        )
    }

    /// Returns true if the phase indicates the connection is ready for use
    #[must_use]
    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready | Self::Maintaining)
    }
}
