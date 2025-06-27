use renik::{Error, WifiConfig};

#[test]
fn test_wifi_config_creation() {
    let config = WifiConfig::new(b"TestNetwork", b"password123").unwrap();

    assert!(config.is_valid());
    assert_eq!(config.get_ssid(), b"TestNetwork");
    assert_eq!(config.get_password(), b"password123");
}

#[test]
fn test_wifi_config_empty_credentials() {
    let config = WifiConfig::new(b"", b"").unwrap();

    assert!(config.is_valid());
    assert_eq!(config.get_ssid(), b"");
    assert_eq!(config.get_password(), b"");
}

#[test]
fn test_wifi_config_ssid_too_long() {
    // SSID longer than 32 bytes
    let long_ssid =
        b"This_is_a_very_long_SSID_name_that_exceeds_the_32_byte_limit_for_WiFi_networks";

    match WifiConfig::new(long_ssid, b"password") {
        Err(Error::CredentialLengthExceeded) => {} // Expected
        _ => panic!("Should have returned CredentialLengthExceeded error"),
    }
}

#[test]
fn test_wifi_config_password_too_long() {
    // Password longer than 64 bytes
    let long_password = b"This_is_a_very_long_password_that_definitely_exceeds_the_64_byte_limit_for_WiFi_network_passwords_and_should_fail";

    match WifiConfig::new(b"TestNetwork", long_password) {
        Err(Error::CredentialLengthExceeded) => {} // Expected
        _ => panic!("Should have returned CredentialLengthExceeded error"),
    }
}

#[test]
fn test_wifi_config_max_valid_lengths() {
    // Test maximum valid lengths (32 bytes for SSID, 64 bytes for password)
    let max_ssid = vec![b'S'; 32];
    let max_password = vec![b'P'; 64];

    let config = WifiConfig::new(&max_ssid, &max_password).unwrap();
    assert!(config.is_valid());
    assert_eq!(config.get_ssid().len(), 32);
    assert_eq!(config.get_password().len(), 64);
}

#[test]
fn test_wifi_config_set_credentials() {
    let mut config = WifiConfig::new(b"Initial", b"initial_pass").unwrap();

    // Test updating credentials
    config
        .set_credentials(b"NewNetwork", b"new_password_123")
        .unwrap();
    assert_eq!(config.get_ssid(), b"NewNetwork");
    assert_eq!(config.get_password(), b"new_password_123");
}

#[test]
fn test_wifi_config_set_ssid_too_long() {
    let mut config = WifiConfig::new(b"Initial", b"initial_pass").unwrap();
    let long_ssid = vec![b'X'; 33]; // 33 bytes, exceeds limit

    assert!(matches!(
        config.set_credentials(&long_ssid, b"valid_password"),
        Err(Error::CredentialLengthExceeded)
    ));

    // Original SSID should be unchanged
    assert_eq!(config.get_ssid(), b"Initial");
}

#[test]
fn test_wifi_config_set_password_too_long() {
    let mut config = WifiConfig::new(b"TestNetwork", b"initial_pass").unwrap();
    let long_password = vec![b'X'; 65]; // 65 bytes, exceeds limit

    assert!(matches!(
        config.set_credentials(b"TestNetwork", &long_password),
        Err(Error::CredentialLengthExceeded)
    ));

    // Original password should be unchanged
    assert_eq!(config.get_password(), b"initial_pass");
}

#[test]
fn test_wifi_config_set_credentials_together() {
    let mut config = WifiConfig::new(b"Initial", b"initial_pass").unwrap();

    config
        .set_credentials(b"HomeNetwork", b"home_password_456")
        .unwrap();
    assert_eq!(config.get_ssid(), b"HomeNetwork");
    assert_eq!(config.get_password(), b"home_password_456");
}

#[test]
fn test_wifi_config_set_credentials_ssid_too_long() {
    let mut config = WifiConfig::new(b"Initial", b"initial_pass").unwrap();
    let long_ssid = vec![b'X'; 33];

    assert!(matches!(
        config.set_credentials(&long_ssid, b"valid_password"),
        Err(Error::CredentialLengthExceeded)
    ));

    // Original credentials should be unchanged
    assert_eq!(config.get_ssid(), b"Initial");
    assert_eq!(config.get_password(), b"initial_pass");
}

#[test]
fn test_wifi_config_set_credentials_password_too_long() {
    let mut config = WifiConfig::new(b"Initial", b"initial_pass").unwrap();
    let long_password = vec![b'X'; 65];

    assert!(matches!(
        config.set_credentials(b"ValidSSID", &long_password),
        Err(Error::CredentialLengthExceeded)
    ));

    // Original credentials should be unchanged
    assert_eq!(config.get_ssid(), b"Initial");
    assert_eq!(config.get_password(), b"initial_pass");
}

#[test]
fn test_wifi_config_special_characters() {
    // Test with special characters in SSID and password
    let ssid_with_spaces = b"My Home Network 2.4GHz";
    let password_with_symbols = b"P@ssw0rd!#$%^&*()";

    let config = WifiConfig::new(ssid_with_spaces, password_with_symbols).unwrap();
    assert!(config.is_valid());
    assert_eq!(config.get_ssid(), ssid_with_spaces);
    assert_eq!(config.get_password(), password_with_symbols);
}

#[test]
fn test_wifi_config_unicode_handling() {
    // Test with non-ASCII characters (UTF-8 encoded)
    let unicode_ssid = "Café-WiFi".as_bytes(); // Contains é character
    let unicode_password = "contraseña123".as_bytes(); // Contains ñ character

    if unicode_ssid.len() <= 32 && unicode_password.len() <= 64 {
        let config = WifiConfig::new(unicode_ssid, unicode_password).unwrap();
        assert!(config.is_valid());
        assert_eq!(config.get_ssid(), unicode_ssid);
        assert_eq!(config.get_password(), unicode_password);
    }
}

#[test]
fn test_wifi_config_default() {
    let config = WifiConfig::default();

    assert!(config.is_valid());
    assert_eq!(config.get_ssid(), b"");
    assert_eq!(config.get_password(), b"");
}

#[test]
fn test_wifi_config_memory_layout() {
    // Ensure the structure has the expected size for embedded use
    let expected_size = 4 + 32 + 1 + 64 + 1 + 2; // magic + ssid + ssid_len + password + password_len + padding
    assert_eq!(core::mem::size_of::<WifiConfig>(), expected_size);

    // Ensure proper alignment
    assert_eq!(core::mem::align_of::<WifiConfig>(), 4);
}

#[test]
fn test_wifi_config_serialization_safety() {
    let config = WifiConfig::new(b"TestNet", b"testpass").unwrap();

    // Test that the structure can be safely converted to bytes
    let config_bytes = unsafe {
        core::slice::from_raw_parts(
            &config as *const _ as *const u8,
            core::mem::size_of::<WifiConfig>(),
        )
    };

    // The first 4 bytes should be the magic number
    let magic_bytes = &config_bytes[0..4];
    let expected_magic = 0x57494649u32.to_le_bytes(); // "WIFI" in little-endian
    assert_eq!(magic_bytes, expected_magic);
}

#[test]
fn test_wifi_config_clear_credentials() {
    let mut config = WifiConfig::new(b"SecretNetwork", b"secret_password").unwrap();

    // Clear credentials by setting empty values
    config.set_credentials(b"", b"").unwrap();
    assert_eq!(config.get_ssid(), b"");
    assert_eq!(config.get_password(), b"");
    assert!(config.is_valid()); // Should still be valid
}

#[test]
fn test_wifi_config_boundary_conditions() {
    // Test exactly at the boundary (32 bytes for SSID)
    let ssid_32_bytes = vec![b'A'; 32];
    let config = WifiConfig::new(&ssid_32_bytes, b"password").unwrap();
    assert_eq!(config.get_ssid().len(), 32);

    // Test exactly at the boundary (64 bytes for password)
    let password_64_bytes = vec![b'B'; 64];
    let config = WifiConfig::new(b"network", &password_64_bytes).unwrap();
    assert_eq!(config.get_password().len(), 64);

    // Test one byte over the boundary should fail
    let ssid_33_bytes = vec![b'C'; 33];
    assert!(matches!(
        WifiConfig::new(&ssid_33_bytes, b"password"),
        Err(Error::CredentialLengthExceeded)
    ));

    let password_65_bytes = vec![b'D'; 65];
    assert!(matches!(
        WifiConfig::new(b"network", &password_65_bytes),
        Err(Error::CredentialLengthExceeded)
    ));
}
