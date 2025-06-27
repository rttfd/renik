use crate::Error;
use bytemuck::{Pod, Zeroable};

/// Magic number used to validate device information structures
/// Value: 0x00444556 (ASCII "DEV")
const DEVICE_INFO_MAGIC: u32 = 0x0044_4556;

/// Device identification and authentication structure
///
/// This structure stores device-specific information including a unique
/// hardware identifier and device secret. It's designed for
/// embedded systems that need to maintain device identity across reboots.
///
/// # Security Note
/// This structure stores sensitive authentication data. Ensure proper
/// memory protection and secure storage mechanisms when persisting this data.
///
/// # Examples
/// ```
/// use renik::DeviceInfo;
///
/// let config = DeviceInfo::new(b"RENIK-01JY1863M2V0S776", b"3854346E44BCB4797450F63E8A252269B9F841EE4065D2F4C8101194AC712A2D7C2B6F6B0C82E953F2F105B5E1048BA706D08412EFB5AC7A58E8C3656A5EDC3E").unwrap();
/// assert!(config.is_valid());
/// ```
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct DeviceInfo {
    /// Magic number for structure validation (0x444556)
    magic: u32, // 4-byte aligned
    /// Unique hardware identifier (16 bytes)
    hardware_id: [u8; 32], // 1-byte aligned
    /// Device secret (128 bytes)
    secret: [u8; 128], // 1-byte aligned
}

impl Default for DeviceInfo {
    /// Creates a new device info structure with default values
    ///
    /// The structure is initialized with the correct magic number
    /// and zeroed identifier/secret fields.
    fn default() -> Self {
        Self {
            magic: DEVICE_INFO_MAGIC,
            hardware_id: [0; 32],
            secret: [0; 128],
        }
    }
}

impl DeviceInfo {
    /// Creates a new `DeviceInfo` instance with the provided hardware ID and secret.
    ///
    /// # Parameters
    /// - `hardware_id`: A byte slice representing the hardware identifier (maximum 16 bytes).
    /// - `secret`: A byte slice representing the device secret (maximum 64 bytes).
    ///
    /// # Returns
    /// - `Ok(DeviceInfo)` if the hardware ID and secret were set successfully.
    /// - `Err(Error)` if the hardware ID or secret length exceeded the maximum allowed.
    ///
    /// # Errors
    /// This method will return an `Error::IdentityLengthExceeded` if either the `hardware_id` or `secret`
    /// parameter exceeds the maximum allowed length.
    pub fn new(hardware_id: &[u8], secret: &[u8]) -> Result<Self, Error> {
        let mut di = Self::default();
        di.set_hardware_id(hardware_id)?;
        di.set_secret(secret)?;
        Ok(di)
    }

    /// Validates the device information structure
    ///
    /// # Returns
    /// - `true` if the magic number is correct
    /// - `false` otherwise
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.magic == DEVICE_INFO_MAGIC
    }

    /// Sets the hardware identifier
    ///
    /// # Parameters
    /// - `hardware_id`: Hardware identifier as byte slice (max 16 bytes)
    ///
    /// # Returns
    /// - `Ok(())` if the hardware ID was set successfully.
    /// - `Err(Error)` if the hardware ID length exceeded the maximum allowed.
    ///
    /// # Errors
    /// Returns `Error::IdentityLengthExceeded` if the hardware ID exceeds 32 bytes.
    ///
    /// # Note
    /// If the input is shorter than 32 bytes, only the specified bytes
    /// are updated, leaving the remainder unchanged.
    pub fn set_hardware_id(&mut self, hardware_id: &[u8]) -> Result<(), Error> {
        if hardware_id.len() > 32 {
            return Err(Error::IdentityLengthExceeded);
        }

        self.hardware_id[..hardware_id.len()].copy_from_slice(hardware_id);
        Ok(())
    }

    /// Sets the device secret
    ///
    /// # Parameters
    /// - `secret`: Device secret as byte slice (max 128 bytes)
    ///
    /// # Returns
    /// - `Ok(())` if the hardware secret was set successfully.
    /// - `Err(Error)` if the hardware secret length exceeded the maximum allowed.
    ///
    /// # Errors
    /// Returns `Error::IdentityLengthExceeded` if the secret exceeds 128 bytes.
    ///
    /// # Note
    /// If the input is shorter than 128 bytes, only the specified bytes
    /// are updated, leaving the remainder unchanged.
    pub fn set_secret(&mut self, secret: &[u8]) -> Result<(), Error> {
        if secret.len() > 128 {
            return Err(Error::IdentityLengthExceeded);
        }

        self.secret[..secret.len()].copy_from_slice(secret);
        Ok(())
    }

    /// Returns the stored hardware identifier
    ///
    /// # Returns
    /// A reference to the complete 32-byte hardware identifier array
    #[must_use]
    pub fn get_hardware_id(&self) -> &[u8] {
        &self.hardware_id
    }

    /// Returns the stored device secret
    ///
    /// # Returns
    /// A reference to the complete 128-byte secret array
    #[must_use]
    pub fn get_secret(&self) -> &[u8] {
        &self.secret
    }
}
