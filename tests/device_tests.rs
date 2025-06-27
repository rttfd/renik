use renik::{DeviceInfo, Error};

#[test]
fn test_device_info_creation() {
    let hardware_id = b"RENIK-01JY1863M2V0S776";
    let secret = b"test_secret_key_123";

    let device = DeviceInfo::new(hardware_id, secret).unwrap();

    assert!(device.is_valid());
    assert_eq!(&device.get_hardware_id()[..hardware_id.len()], hardware_id);
    assert_eq!(&device.get_secret()[..secret.len()], secret);
}

#[test]
fn test_device_info_default() {
    let device = DeviceInfo::default();

    assert!(device.is_valid());
    assert_eq!(device.get_hardware_id(), &[0u8; 32]);
    assert_eq!(device.get_secret(), &[0u8; 128]);
}

#[test]
fn test_device_info_hardware_id_too_long() {
    let long_hardware_id = vec![b'X'; 33]; // 33 bytes, exceeds 32 byte limit
    let secret = b"test_secret";

    match DeviceInfo::new(&long_hardware_id, secret) {
        Err(Error::IdentityLengthExceeded) => {} // Expected
        _ => panic!("Should have returned IdentityLengthExceeded error"),
    }
}

#[test]
fn test_device_info_secret_too_long() {
    let hardware_id = b"RENIK-TEST";
    let long_secret = vec![b'X'; 129]; // 129 bytes, exceeds 128 byte limit

    match DeviceInfo::new(hardware_id, &long_secret) {
        Err(Error::IdentityLengthExceeded) => {} // Expected
        _ => panic!("Should have returned IdentityLengthExceeded error"),
    }
}

#[test]
fn test_device_info_max_valid_lengths() {
    // Test maximum valid lengths (32 bytes for hardware ID, 128 bytes for secret)
    let max_hardware_id = vec![b'H'; 32];
    let max_secret = vec![b'S'; 128];

    let device = DeviceInfo::new(&max_hardware_id, &max_secret).unwrap();
    assert!(device.is_valid());
    assert_eq!(device.get_hardware_id(), &max_hardware_id[..]);
    assert_eq!(device.get_secret(), &max_secret[..]);
}

#[test]
fn test_device_info_set_hardware_id() {
    let mut device = DeviceInfo::default();
    let hardware_id = b"RENIK-NEW-DEVICE-ID";

    device.set_hardware_id(hardware_id).unwrap();
    assert_eq!(&device.get_hardware_id()[..hardware_id.len()], hardware_id);

    // Test setting hardware ID too long
    let long_hardware_id = vec![b'X'; 33];
    assert!(matches!(
        device.set_hardware_id(&long_hardware_id),
        Err(Error::IdentityLengthExceeded)
    ));
}

#[test]
fn test_device_info_set_secret() {
    let mut device = DeviceInfo::default();
    let secret = b"new_secret_key_for_device_authentication";

    device.set_secret(secret).unwrap();
    assert_eq!(&device.get_secret()[..secret.len()], secret);

    // Test setting secret too long
    let long_secret = vec![b'X'; 129];
    assert!(matches!(
        device.set_secret(&long_secret),
        Err(Error::IdentityLengthExceeded)
    ));
}

#[test]
fn test_device_info_empty_fields() {
    let device = DeviceInfo::new(b"", b"").unwrap();
    assert!(device.is_valid());

    // The arrays should still be valid, just with empty content at the start
    assert_eq!(device.get_hardware_id()[0], 0);
    assert_eq!(device.get_secret()[0], 0);
}

#[test]
fn test_device_info_partial_update() {
    let mut device = DeviceInfo::new(b"ORIGINAL-ID", b"original_secret").unwrap();

    // Update only hardware ID
    let new_hardware_id = b"NEW-HARDWARE-ID";
    device.set_hardware_id(new_hardware_id).unwrap();

    // Hardware ID should be updated, but secret should remain
    assert_eq!(
        &device.get_hardware_id()[..new_hardware_id.len()],
        new_hardware_id
    );
    assert_eq!(&device.get_secret()[..15], b"original_secret");

    // Update only secret
    let new_secret = b"new_secret";
    device.set_secret(new_secret).unwrap();

    // Secret should be updated, hardware ID should remain
    assert_eq!(
        &device.get_hardware_id()[..new_hardware_id.len()],
        new_hardware_id
    );
    assert_eq!(&device.get_secret()[..new_secret.len()], new_secret);
}

#[test]
fn test_device_info_memory_layout() {
    // Test that the structure has the expected size for embedded use
    let expected_size = 4 + 32 + 128; // magic + hardware_id + secret
    assert_eq!(core::mem::size_of::<DeviceInfo>(), expected_size);

    // Ensure proper alignment
    assert_eq!(core::mem::align_of::<DeviceInfo>(), 4);
}

#[test]
fn test_device_info_serialization_safety() {
    let device = DeviceInfo::new(b"TEST-DEVICE", b"test_secret").unwrap();

    // Test that the structure can be safely converted to bytes
    let device_bytes = unsafe {
        core::slice::from_raw_parts(
            &device as *const _ as *const u8,
            core::mem::size_of::<DeviceInfo>(),
        )
    };

    // The first 4 bytes should be the magic number
    let magic_bytes = &device_bytes[0..4];
    let expected_magic = 0x00444556u32.to_le_bytes(); // "DEV" in little-endian
    assert_eq!(magic_bytes, expected_magic);
}

#[test]
fn test_device_info_binary_data() {
    // Test with binary data (not just text)
    let hardware_id = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
    let secret = [0xFF, 0xFE, 0xFD, 0xFC, 0xFB, 0xFA, 0xF9, 0xF8];

    let device = DeviceInfo::new(&hardware_id, &secret).unwrap();
    assert!(device.is_valid());
    assert_eq!(&device.get_hardware_id()[..hardware_id.len()], &hardware_id);
    assert_eq!(&device.get_secret()[..secret.len()], &secret);
}

#[test]
fn test_device_info_boundary_conditions() {
    // Test exactly at the boundary (32 bytes for hardware ID)
    let hardware_id_32_bytes = vec![b'A'; 32];
    let device = DeviceInfo::new(&hardware_id_32_bytes, b"secret").unwrap();
    assert_eq!(device.get_hardware_id(), &hardware_id_32_bytes[..]);

    // Test exactly at the boundary (128 bytes for secret)
    let secret_128_bytes = vec![b'B'; 128];
    let device = DeviceInfo::new(b"device", &secret_128_bytes).unwrap();
    assert_eq!(device.get_secret(), &secret_128_bytes[..]);

    // Test one byte over the boundary should fail
    let hardware_id_33_bytes = vec![b'C'; 33];
    assert!(matches!(
        DeviceInfo::new(&hardware_id_33_bytes, b"secret"),
        Err(Error::IdentityLengthExceeded)
    ));

    let secret_129_bytes = vec![b'D'; 129];
    assert!(matches!(
        DeviceInfo::new(b"device", &secret_129_bytes),
        Err(Error::IdentityLengthExceeded)
    ));
}
