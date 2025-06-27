use thiserror_no_std::Error;

/// Error type for configuration-related operations
#[derive(Debug, Error)]
pub enum Error {
    /// SSID or password length exceeded the maximum allowed
    #[error("SSID or password length exceeded the maximum allowed")]
    CredentialLengthExceeded,
    /// Hardware ID or secret length exceeded the maximum allowed
    #[error("Hardware ID or secret length exceeded the maximum allowed")]
    IdentityLengthExceeded,
    /// Invalid Bluetooth device name or pairing key length
    #[error("Invalid Bluetooth device name or pairing key length")]
    InvalidBluetoothDeviceInfo,
    /// Bluetooth device list is full
    #[error("Bluetooth device list is full")]
    DeviceListFull,
    /// Index out of bounds
    #[error("Index out of bounds")]
    IndexOutOfBounds,
}
