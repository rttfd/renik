use crate::Error;
use bytemuck::{Pod, Zeroable};

/// Magic number used to validate Wi-Fi configuration structures
/// Value: 0x57494649 (ASCII "WIFI")
const WIFI_CONFIG_MAGIC: u32 = 0x5749_4649;

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
/// let config = WifiConfig::new(b"MyNetwork", b"password123").unwrap();
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
    /// Creates a new Wi-Fi configuration with the provided SSID and password
    ///
    /// # Parameters
    /// - `ssid`: Network name as byte slice (max 32 bytes)
    /// - `password`: Network password as byte slice (max 64 bytes)
    ///
    /// # Returns
    /// - `Ok(WifiConfig)` if the credentials were set successfully
    /// - `Err(Error)` if the SSID or password length exceeded the maximum allowed
    ///
    /// # Errors
    /// Returns `Error::CredentialLengthExceeded` if either the SSID exceeds 32 bytes
    /// or the password exceeds 64 bytes.
    pub fn new(ssid: &[u8], password: &[u8]) -> Result<Self, Error> {
        let mut wf = Self::default();
        wf.set_credentials(ssid, password)?;
        Ok(wf)
    }

    /// Validates the Wi-Fi configuration structure
    ///
    /// # Returns
    /// - `true` if the magic number is correct
    /// - `false` otherwise
    ///
    /// # Note
    /// This method only checks for basic structural validity, not the
    /// actual correctness of the Wi-Fi credentials.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.magic == WIFI_CONFIG_MAGIC
    }

    /// Sets the Wi-Fi network credentials
    ///
    /// # Parameters
    /// - `ssid`: Network name as byte slice (max 32 bytes)
    /// - `password`: Network password as byte slice (max 64 bytes)
    ///
    /// # Returns
    /// - `Ok(())` if the credentials were set successfully
    /// - `Err(Error)` if the SSID or password length exceeded the maximum allowed
    ///
    /// # Errors
    /// Returns `Error::CredentialLengthExceeded` if either the SSID exceeds 32 bytes
    /// or the password exceeds 64 bytes.
    ///
    /// # Behavior
    /// - Clears existing credential buffers before setting new values
    /// - Updates length fields to reflect actual credential sizes
    /// - Pads unused buffer space with zeros
    #[allow(clippy::cast_possible_truncation)]
    pub fn set_credentials(&mut self, ssid: &[u8], password: &[u8]) -> Result<(), Error> {
        if ssid.len() > 32 || password.len() > 64 {
            return Err(Error::CredentialLengthExceeded);
        }

        // Safe cast: we've already validated the lengths are within u8 range
        self.ssid_len = ssid.len() as u8;
        self.password_len = password.len() as u8;

        // Clear buffers to ensure no residual data
        self.ssid.fill(0);
        self.password.fill(0);

        // Copy new credentials into buffers
        self.ssid[..ssid.len()].copy_from_slice(ssid);
        self.password[..password.len()].copy_from_slice(password);

        Ok(())
    }

    /// Returns the stored SSID as a byte slice
    ///
    /// # Returns
    /// A slice containing only the valid SSID bytes (length determined by `ssid_len`)
    #[must_use]
    pub fn get_ssid(&self) -> &[u8] {
        &self.ssid[..self.ssid_len as usize]
    }

    /// Returns the stored password as a byte slice
    ///
    /// # Returns
    /// A slice containing only the valid password bytes (length determined by `password_len`)
    #[must_use]
    pub fn get_password(&self) -> &[u8] {
        &self.password[..self.password_len as usize]
    }
}
