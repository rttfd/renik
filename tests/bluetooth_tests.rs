use renik::{
    BluetoothConnectionParams, BluetoothConnectionPhase, BluetoothConnectionState,
    BluetoothDeviceInfo, BluetoothDeviceList, BluetoothSecurityInfo, ConnHandle, Error,
};

#[test]
fn test_bluetooth_device_info_creation() {
    let mac_addr = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC];
    let device = BluetoothDeviceInfo::new(&mac_addr, b"Test Device").unwrap();

    assert!(device.is_valid());
    assert_eq!(device.get_mac_address(), &mac_addr);
    assert_eq!(device.get_device_name(), b"Test Device");
    assert_eq!(
        device.get_device_type(),
        BluetoothDeviceInfo::DEVICE_TYPE_UNKNOWN
    );
    assert!(!device.is_paired());
    assert!(!device.is_connected());
    assert!(!device.is_trusted());
}

#[test]
fn test_bluetooth_device_info_name_too_long() {
    let mac_addr = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC];
    let long_name =
        b"This device name is way too long and exceeds the 32 byte limit for device names";

    match BluetoothDeviceInfo::new(&mac_addr, long_name) {
        Err(Error::InvalidBluetoothDeviceInfo) => {} // Expected
        _ => panic!("Should have returned InvalidBluetoothDeviceInfo error"),
    }
}

#[test]
fn test_bluetooth_device_info_pairing_key() {
    let mac_addr = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC];
    let mut device = BluetoothDeviceInfo::new(&mac_addr, b"Test Device").unwrap();

    let pairing_key = b"test_key_123";
    device.set_pairing_key(pairing_key).unwrap();
    assert_eq!(device.get_pairing_key(), pairing_key);

    // Test pairing key too long
    let long_key = vec![b'x'; 65]; // 65 bytes, exceeds 64 byte limit
    assert!(matches!(
        device.set_pairing_key(&long_key),
        Err(Error::InvalidBluetoothDeviceInfo)
    ));
}

#[test]
fn test_bluetooth_device_info_flags() {
    let mac_addr = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC];
    let mut device = BluetoothDeviceInfo::new(&mac_addr, b"Test Device").unwrap();

    // Test individual flags
    device.add_flag(BluetoothDeviceInfo::FLAG_PAIRED);
    assert!(device.is_paired());
    assert!(device.has_flag(BluetoothDeviceInfo::FLAG_PAIRED));

    device.add_flag(BluetoothDeviceInfo::FLAG_TRUSTED);
    assert!(device.is_trusted());

    device.add_flag(BluetoothDeviceInfo::FLAG_AUTO_RECONNECT);
    assert!(device.supports_auto_reconnect());

    // Test removing flags
    device.remove_flag(BluetoothDeviceInfo::FLAG_PAIRED);
    assert!(!device.is_paired());
    assert!(device.is_trusted()); // Should still be trusted
}

#[test]
fn test_bluetooth_device_info_class_of_device() {
    let mac_addr = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC];
    let mut device = BluetoothDeviceInfo::new(&mac_addr, b"Audio Device").unwrap();

    // Set audio device class - major class 4 (audio) in bits 2-6 of second byte
    let audio_class = [0x04, 0x10, 0x24]; // 0x10 = 0b00010000, bits 2-6 = 4 (audio)
    device.set_class_of_device(&audio_class);

    assert_eq!(device.get_class_of_device(), &audio_class);
    assert_eq!(
        device.get_device_type(),
        BluetoothDeviceInfo::DEVICE_TYPE_AUDIO
    );
}

#[test]
fn test_bluetooth_device_list() {
    let mut device_list = BluetoothDeviceList::default();
    assert!(device_list.is_empty());
    assert_eq!(device_list.len(), 0);

    // Add first device
    let mac_addr1 = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC];
    let device1 = BluetoothDeviceInfo::new(&mac_addr1, b"Device 1").unwrap();
    device_list.add_device(device1).unwrap();

    assert!(!device_list.is_empty());
    assert_eq!(device_list.len(), 1);

    // Add second device
    let mac_addr2 = [0x98, 0x76, 0x54, 0x32, 0x10, 0xFE];
    let device2 = BluetoothDeviceInfo::new(&mac_addr2, b"Device 2").unwrap();
    device_list.add_device(device2).unwrap();

    assert_eq!(device_list.len(), 2);

    // Test retrieving devices
    let retrieved_device1 = device_list.get_device(0).unwrap();
    assert_eq!(retrieved_device1.get_device_name(), b"Device 1");

    let retrieved_device2 = device_list.get_device(1).unwrap();
    assert_eq!(retrieved_device2.get_device_name(), b"Device 2");

    // Test out of bounds access
    assert!(matches!(
        device_list.get_device(2),
        Err(Error::IndexOutOfBounds)
    ));
}

#[test]
fn test_bluetooth_device_list_remove() {
    let mut device_list = BluetoothDeviceList::default();

    // Add three devices
    for i in 0..3 {
        let mac_addr = [0x10 + i, 0x20, 0x30, 0x40, 0x50, 0x60];
        let name = format!("Device {}", i);
        let device = BluetoothDeviceInfo::new(&mac_addr, name.as_bytes()).unwrap();
        device_list.add_device(device).unwrap();
    }

    assert_eq!(device_list.len(), 3);

    // Remove middle device
    device_list.remove_device(1).unwrap();
    assert_eq!(device_list.len(), 2);

    // Check that devices shifted correctly
    let device0 = device_list.get_device(0).unwrap();
    assert_eq!(device0.get_device_name(), b"Device 0");

    let device1 = device_list.get_device(1).unwrap();
    assert_eq!(device1.get_device_name(), b"Device 2");

    // Test removing out of bounds
    assert!(matches!(
        device_list.remove_device(5),
        Err(Error::IndexOutOfBounds)
    ));
}

#[test]
fn test_bluetooth_device_list_full() {
    let mut device_list = BluetoothDeviceList::default();

    // Fill the list to capacity (10 devices)
    for i in 0..10 {
        let mac_addr = [i as u8, 0x20, 0x30, 0x40, 0x50, 0x60];
        let name = format!("Device {}", i);
        let device = BluetoothDeviceInfo::new(&mac_addr, name.as_bytes()).unwrap();
        device_list.add_device(device).unwrap();
    }

    assert_eq!(device_list.len(), 10);

    // Try to add one more device (should fail)
    let mac_addr = [0xFF, 0x20, 0x30, 0x40, 0x50, 0x60];
    let device = BluetoothDeviceInfo::new(&mac_addr, b"Extra Device").unwrap();

    assert!(matches!(
        device_list.add_device(device),
        Err(Error::DeviceListFull)
    ));
}

#[test]
fn test_conn_handle() {
    // Test valid handle creation
    let handle = ConnHandle::new(0x0001);
    assert_eq!(handle.raw(), 0x0001);

    // Test maximum valid handle
    let max_handle = ConnHandle::new(0x0EFF);
    assert_eq!(max_handle.raw(), 0x0EFF);

    // Test default handle
    let default_handle = ConnHandle::default();
    assert_eq!(default_handle.raw(), 0x0000);

    // Test From trait implementations
    let handle_from_u16: ConnHandle = 0x0042.into();
    assert_eq!(handle_from_u16.raw(), 0x0042);

    let u16_from_handle: u16 = handle_from_u16.into();
    assert_eq!(u16_from_handle, 0x0042);
}

#[test]
#[should_panic(expected = "Connection handle must be <= 0x0EFF")]
fn test_conn_handle_invalid() {
    let _ = ConnHandle::new(0x0F00); // Should panic
}

#[test]
fn test_bluetooth_connection_phase() {
    // Test default
    let default_phase = BluetoothConnectionPhase::default();
    assert_eq!(default_phase, BluetoothConnectionPhase::Idle);

    // Test is_connected
    assert!(!BluetoothConnectionPhase::Idle.is_connected());
    assert!(!BluetoothConnectionPhase::Discovery.is_connected());
    assert!(!BluetoothConnectionPhase::Connecting.is_connected());
    assert!(BluetoothConnectionPhase::Connected.is_connected());
    assert!(BluetoothConnectionPhase::Authenticating.is_connected());
    assert!(BluetoothConnectionPhase::Ready.is_connected());

    // Test is_secure
    assert!(!BluetoothConnectionPhase::Connected.is_secure());
    assert!(!BluetoothConnectionPhase::Authenticating.is_secure());
    assert!(BluetoothConnectionPhase::FullyConnected.is_secure());
    assert!(BluetoothConnectionPhase::Ready.is_secure());

    // Test is_ready
    assert!(!BluetoothConnectionPhase::Connected.is_ready());
    assert!(!BluetoothConnectionPhase::FullyConnected.is_ready());
    assert!(BluetoothConnectionPhase::Ready.is_ready());
    assert!(BluetoothConnectionPhase::Maintaining.is_ready());
}

#[test]
fn test_bluetooth_connection_state_fsm() {
    let mut connection_state = BluetoothConnectionState::default();

    // Test initial state
    assert_eq!(
        connection_state.get_connection_phase(),
        BluetoothConnectionPhase::Idle
    );

    // Test valid transitions from Idle
    assert!(connection_state.advance_to_phase(BluetoothConnectionPhase::Discovery));
    assert_eq!(
        connection_state.get_connection_phase(),
        BluetoothConnectionPhase::Discovery
    );

    assert!(connection_state.advance_to_phase(BluetoothConnectionPhase::Connecting));
    assert_eq!(
        connection_state.get_connection_phase(),
        BluetoothConnectionPhase::Connecting
    );

    // Test invalid transition
    assert!(!connection_state.advance_to_phase(BluetoothConnectionPhase::Ready));
    assert_eq!(
        connection_state.get_connection_phase(),
        BluetoothConnectionPhase::Connecting
    );

    // Continue with valid transitions
    assert!(connection_state.advance_to_phase(BluetoothConnectionPhase::Connected));
    assert!(connection_state.advance_to_phase(BluetoothConnectionPhase::Authenticating));
    assert!(connection_state.advance_to_phase(BluetoothConnectionPhase::SettingUpEncryption));
    assert!(connection_state.advance_to_phase(BluetoothConnectionPhase::FullyConnected));
    assert!(connection_state.advance_to_phase(BluetoothConnectionPhase::Ready));

    // Test that any phase can transition to Idle
    assert!(connection_state.advance_to_phase(BluetoothConnectionPhase::Idle));
    assert_eq!(
        connection_state.get_connection_phase(),
        BluetoothConnectionPhase::Idle
    );
}

#[test]
fn test_bluetooth_connection_state_error_recovery() {
    let mut connection_state = BluetoothConnectionState::default();

    // Go to connected state and then to authenticating
    connection_state.advance_to_phase(BluetoothConnectionPhase::Connecting);
    connection_state.advance_to_phase(BluetoothConnectionPhase::Connected);
    connection_state.advance_to_phase(BluetoothConnectionPhase::Authenticating);

    // Test failure transition from authenticating state
    assert!(connection_state.advance_to_phase(BluetoothConnectionPhase::Failed));
    assert_eq!(
        connection_state.get_connection_phase(),
        BluetoothConnectionPhase::Failed
    );

    // Test reconnection from failed state
    assert!(connection_state.advance_to_phase(BluetoothConnectionPhase::Reconnecting));
    assert!(connection_state.advance_to_phase(BluetoothConnectionPhase::Connecting));
    assert!(connection_state.advance_to_phase(BluetoothConnectionPhase::Connected));
}

#[test]
fn test_bluetooth_connection_state_basic_functionality() {
    let mut connection_state = BluetoothConnectionState::default();

    // Test setting remote device
    let mac_addr = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC];
    let device = BluetoothDeviceInfo::new(&mac_addr, b"Test Device").unwrap();
    connection_state.set_remote_device(device);

    let remote_device = connection_state.get_remote_device();
    assert_eq!(remote_device.get_device_name(), b"Test Device");

    // Test connection status
    assert!(!connection_state.is_connected());
    connection_state.set_connected(true);
    assert!(connection_state.is_connected());

    // Test authentication status
    assert!(!connection_state.is_authenticated());
    connection_state.set_authenticated(true);
    assert!(connection_state.is_authenticated());

    // Test link quality
    connection_state.set_link_quality(85);
    assert_eq!(connection_state.get_link_quality(), 85);

    // Test connection handle
    let handle = ConnHandle::new(0x0042);
    connection_state.set_connection_handle(Some(handle));
    assert_eq!(connection_state.get_connection_handle(), Some(handle));

    // Test clearing connection handle
    connection_state.set_connection_handle(None);
    assert_eq!(connection_state.get_connection_handle(), None);
}

#[test]
fn test_bluetooth_connection_params() {
    let mut params = BluetoothConnectionParams::default();

    // Test default values
    assert_eq!(params.connection_handle.raw(), 0);
    assert_eq!(params.connection_interval, 0);
    assert_eq!(params.rssi, -127);

    // Test setting values
    params.connection_handle = ConnHandle::new(0x0001);
    params.connection_interval = 24; // 30ms intervals
    params.connection_latency = 0;
    params.supervision_timeout = 200; // 2000ms timeout
    params.rssi = -45;

    assert_eq!(params.connection_handle.raw(), 0x0001);
    assert_eq!(params.connection_interval, 24);
    assert_eq!(params.rssi, -45);
}

#[test]
fn test_bluetooth_security_info() {
    let mut security = BluetoothSecurityInfo::default();

    // Test default values
    assert_eq!(security.security_level, 1);
    assert_eq!(security.authenticated, 0);
    assert_eq!(security.encrypted, 0);

    // Test setting security information
    security.link_key = [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
        0x10,
    ];
    security.link_key_valid = 1;
    security.authenticated = 1;
    security.encrypted = 1;
    security.security_level = 4;

    assert_eq!(security.link_key[0], 0x01);
    assert_eq!(security.link_key[15], 0x10);
    assert_eq!(security.authenticated, 1);
    assert_eq!(security.encrypted, 1);
    assert_eq!(security.security_level, 4);
}

#[test]
fn test_bluetooth_device_info_connection_params_update() {
    let mac_addr = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC];
    let mut device = BluetoothDeviceInfo::new(&mac_addr, b"Test Device").unwrap();

    let mut params = BluetoothConnectionParams::default();
    params.connection_handle = ConnHandle::new(0x0042);
    params.rssi = -50;

    device.update_connection_params(&params);

    let device_params = device.get_connection_params();
    assert_eq!(device_params.connection_handle.raw(), 0x0042);
    assert_eq!(device_params.rssi, -50);
    assert!(device.is_connected()); // Should set connected flag
}

#[test]
fn test_bluetooth_device_info_security_info_update() {
    let mac_addr = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC];
    let mut device = BluetoothDeviceInfo::new(&mac_addr, b"Test Device").unwrap();

    let mut security = BluetoothSecurityInfo::default();
    security.authenticated = 1;
    security.encrypted = 1;

    device.update_security_info(&security);

    let device_security = device.get_security_info();
    assert_eq!(device_security.authenticated, 1);
    assert_eq!(device_security.encrypted, 1);
    assert!(device.is_paired()); // Should set paired flag when authenticated
}

#[test]
fn test_memory_layout_alignment() {
    // Test that structures have expected sizes for embedded use
    assert_eq!(core::mem::size_of::<ConnHandle>(), 2);
    assert_eq!(core::mem::size_of::<BluetoothConnectionPhase>(), 1);

    // Ensure proper alignment
    assert_eq!(core::mem::align_of::<BluetoothDeviceInfo>(), 4);
    assert_eq!(core::mem::align_of::<BluetoothDeviceList>(), 4);
    assert_eq!(core::mem::align_of::<BluetoothConnectionState>(), 4);
}

#[test]
fn test_bluetooth_fsm_all_valid_transitions() {
    let mut connection_state = BluetoothConnectionState::default();

    // Test complete successful connection flow
    assert!(connection_state.advance_to_phase(BluetoothConnectionPhase::Discovery));
    assert!(connection_state.advance_to_phase(BluetoothConnectionPhase::Connecting));
    assert!(connection_state.advance_to_phase(BluetoothConnectionPhase::Connected));
    assert!(connection_state.advance_to_phase(BluetoothConnectionPhase::Authenticating));
    assert!(connection_state.advance_to_phase(BluetoothConnectionPhase::SettingUpEncryption));
    assert!(connection_state.advance_to_phase(BluetoothConnectionPhase::FullyConnected));
    assert!(connection_state.advance_to_phase(BluetoothConnectionPhase::ServiceDiscovery));
    assert!(connection_state.advance_to_phase(BluetoothConnectionPhase::Ready));
    assert!(connection_state.advance_to_phase(BluetoothConnectionPhase::Maintaining));

    // Test disconnect flow
    assert!(connection_state.advance_to_phase(BluetoothConnectionPhase::Disconnecting));
    assert!(connection_state.advance_to_phase(BluetoothConnectionPhase::Idle));
}

#[test]
fn test_bluetooth_fsm_invalid_transitions() {
    let mut connection_state = BluetoothConnectionState::default();

    // Test invalid transitions from each state
    assert!(!connection_state.advance_to_phase(BluetoothConnectionPhase::Connected)); // Can't go directly from Idle to Connected
    assert!(!connection_state.advance_to_phase(BluetoothConnectionPhase::Ready)); // Can't go directly from Idle to Ready

    connection_state.advance_to_phase(BluetoothConnectionPhase::Discovery);
    assert!(!connection_state.advance_to_phase(BluetoothConnectionPhase::Connected)); // Can't skip Connecting
    assert!(!connection_state.advance_to_phase(BluetoothConnectionPhase::Ready)); // Can't skip multiple phases

    connection_state.advance_to_phase(BluetoothConnectionPhase::Connecting);
    assert!(!connection_state.advance_to_phase(BluetoothConnectionPhase::Authenticating)); // Can't skip Connected
    assert!(!connection_state.advance_to_phase(BluetoothConnectionPhase::Ready)); // Can't skip multiple phases
}

#[test]
fn test_bluetooth_fsm_emergency_transitions() {
    let mut connection_state = BluetoothConnectionState::default();

    // Test that any state can transition to Idle (emergency reset)
    for phase in [
        BluetoothConnectionPhase::Discovery,
        BluetoothConnectionPhase::Connecting,
        BluetoothConnectionPhase::Connected,
        BluetoothConnectionPhase::Authenticating,
        BluetoothConnectionPhase::SettingUpEncryption,
        BluetoothConnectionPhase::FullyConnected,
        BluetoothConnectionPhase::ServiceDiscovery,
        BluetoothConnectionPhase::Ready,
        BluetoothConnectionPhase::Maintaining,
        BluetoothConnectionPhase::Reconnecting,
        BluetoothConnectionPhase::Failed,
        BluetoothConnectionPhase::Disconnecting,
    ] {
        connection_state.set_connection_phase(phase);
        assert!(connection_state.advance_to_phase(BluetoothConnectionPhase::Idle));
        assert_eq!(
            connection_state.get_connection_phase(),
            BluetoothConnectionPhase::Idle
        );
    }
}

#[test]
fn test_bluetooth_fsm_alternate_paths() {
    let mut connection_state = BluetoothConnectionState::default();

    // Test direct service discovery path (skipping authentication)
    connection_state.advance_to_phase(BluetoothConnectionPhase::Connecting);
    connection_state.advance_to_phase(BluetoothConnectionPhase::Connected);
    assert!(connection_state.advance_to_phase(BluetoothConnectionPhase::ServiceDiscovery));
    assert!(connection_state.advance_to_phase(BluetoothConnectionPhase::Ready));

    // Test disconnect from various states
    connection_state.set_connection_phase(BluetoothConnectionPhase::Authenticating);
    assert!(connection_state.advance_to_phase(BluetoothConnectionPhase::Disconnecting));

    connection_state.set_connection_phase(BluetoothConnectionPhase::SettingUpEncryption);
    assert!(connection_state.advance_to_phase(BluetoothConnectionPhase::Disconnecting));

    connection_state.set_connection_phase(BluetoothConnectionPhase::FullyConnected);
    assert!(connection_state.advance_to_phase(BluetoothConnectionPhase::Disconnecting));
}

#[test]
fn test_bluetooth_device_info_comprehensive_flags() {
    let mac_addr = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC];
    let mut device = BluetoothDeviceInfo::new(&mac_addr, b"Test Device").unwrap();

    // Test all flag combinations
    let all_flags = [
        BluetoothDeviceInfo::FLAG_PAIRED,
        BluetoothDeviceInfo::FLAG_TRUSTED,
        BluetoothDeviceInfo::FLAG_AUDIO,
        BluetoothDeviceInfo::FLAG_INPUT,
        BluetoothDeviceInfo::FLAG_FILE_TRANSFER,
        BluetoothDeviceInfo::FLAG_CONNECTED,
        BluetoothDeviceInfo::FLAG_AUTO_RECONNECT,
        BluetoothDeviceInfo::FLAG_RECENTLY_DISCOVERED,
    ];

    // Test setting and checking each flag individually
    for &flag in &all_flags {
        device.add_flag(flag);
        assert!(device.has_flag(flag));
    }

    // Test that all flags are set
    let combined_flags = all_flags.iter().fold(0u8, |acc, &flag| acc | flag);
    assert_eq!(device.get_flags(), combined_flags);

    // Test removing flags
    for &flag in &all_flags {
        device.remove_flag(flag);
        assert!(!device.has_flag(flag));
    }

    assert_eq!(device.get_flags(), 0);
}

#[test]
fn test_bluetooth_device_info_all_device_types() {
    let mac_addr = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC];
    let mut device = BluetoothDeviceInfo::new(&mac_addr, b"Test Device").unwrap();

    // Test all device type classifications
    let device_types = [
        (1, BluetoothDeviceInfo::DEVICE_TYPE_COMPUTER),
        (2, BluetoothDeviceInfo::DEVICE_TYPE_PHONE),
        (3, BluetoothDeviceInfo::DEVICE_TYPE_NETWORK),
        (4, BluetoothDeviceInfo::DEVICE_TYPE_AUDIO),
        (5, BluetoothDeviceInfo::DEVICE_TYPE_PERIPHERAL),
        (6, BluetoothDeviceInfo::DEVICE_TYPE_IMAGING),
        (7, BluetoothDeviceInfo::DEVICE_TYPE_WEARABLE),
        (8, BluetoothDeviceInfo::DEVICE_TYPE_TOY),
    ];

    for (major_class, expected_type) in device_types {
        // Create class of device with specific major class
        let class_bytes = [0x00, (major_class << 2) as u8, 0x00];
        device.set_class_of_device(&class_bytes);
        assert_eq!(device.get_device_type(), expected_type);
    }

    // Test unknown device type
    let unknown_class = [0x00, 0xFC, 0x00]; // Major class 63 (invalid)
    device.set_class_of_device(&unknown_class);
    assert_eq!(
        device.get_device_type(),
        BluetoothDeviceInfo::DEVICE_TYPE_UNKNOWN
    );
}

#[test]
fn test_bluetooth_connection_params_comprehensive() {
    let mut params = BluetoothConnectionParams::default();

    // Test all parameter ranges
    params.connection_handle = ConnHandle::new(0x0EFF); // Maximum valid handle
    params.connection_interval = 3200; // Maximum interval
    params.connection_latency = 499; // Maximum latency
    params.supervision_timeout = 3200; // Maximum timeout
    params.master_clock_accuracy = 7; // Maximum accuracy
    params.link_type = 0x02; // SCO link
    params.encryption_enabled = 0x01; // Enabled
    params.rssi = 127; // Maximum RSSI
    params.connected_at = u32::MAX; // Maximum timestamp
    params.last_activity = u32::MAX; // Maximum timestamp

    // Verify all values are set correctly
    assert_eq!(params.connection_handle.raw(), 0x0EFF);
    assert_eq!(params.connection_interval, 3200);
    assert_eq!(params.connection_latency, 499);
    assert_eq!(params.supervision_timeout, 3200);
    assert_eq!(params.master_clock_accuracy, 7);
    assert_eq!(params.link_type, 0x02);
    assert_eq!(params.encryption_enabled, 0x01);
    assert_eq!(params.rssi, 127);
    assert_eq!(params.connected_at, u32::MAX);
    assert_eq!(params.last_activity, u32::MAX);
}

#[test]
fn test_bluetooth_security_info_comprehensive() {
    let mut security = BluetoothSecurityInfo::default();

    // Test all security parameters
    security.link_key = [0xFF; 16]; // Maximum key
    security.link_key_type = 0x07; // Maximum type
    security.auth_requirements = 0xFF; // All requirements
    security.io_capabilities = 0x04; // Maximum capabilities
    security.security_level = 0x04; // Maximum level
    security.pin_length = 16; // Maximum PIN length
    security.link_key_valid = 1;
    security.authenticated = 1;
    security.encrypted = 1;
    security.ssp_supported = 1;
    security.mitm_required = 1;

    // Verify all values are set correctly
    assert_eq!(security.link_key, [0xFF; 16]);
    assert_eq!(security.link_key_type, 0x07);
    assert_eq!(security.auth_requirements, 0xFF);
    assert_eq!(security.io_capabilities, 0x04);
    assert_eq!(security.security_level, 0x04);
    assert_eq!(security.pin_length, 16);
    assert_eq!(security.link_key_valid, 1);
    assert_eq!(security.authenticated, 1);
    assert_eq!(security.encrypted, 1);
    assert_eq!(security.ssp_supported, 1);
    assert_eq!(security.mitm_required, 1);
}

#[test]
fn test_bluetooth_device_info_timestamps() {
    let mac_addr = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC];
    let mut device = BluetoothDeviceInfo::new(&mac_addr, b"Test Device").unwrap();

    // Test timestamp operations
    let now = 1234567890u32;
    device.update_last_seen(now);
    device.update_last_connected(now - 3600); // 1 hour ago
    device.set_connection_count(5);

    // Test increment operations
    device.increment_connection_count();
    assert_eq!(device.get_connection_params().connection_handle.raw(), 0); // Default handle

    // Test setting individual timestamps
    device.set_last_seen(now + 100);
    device.set_last_connected(now + 50);

    // Verify operations don't interfere with each other
    assert!(device.is_valid());
    assert_eq!(device.get_device_name(), b"Test Device");
}

#[test]
fn test_conn_handle_edge_cases() {
    // Test minimum valid handle
    let min_handle = ConnHandle::new(0x0000);
    assert_eq!(min_handle.raw(), 0x0000);

    // Test maximum valid handle
    let max_handle = ConnHandle::new(0x0EFF);
    assert_eq!(max_handle.raw(), 0x0EFF);

    // Test conversion consistency
    for i in 0..=0x0EFF {
        let handle = ConnHandle::new(i);
        let converted: u16 = handle.into();
        let back_converted = ConnHandle::from(converted);
        assert_eq!(handle, back_converted);
        assert_eq!(converted, i);
    }
}

#[test]
#[should_panic(expected = "Connection handle must be <= 0x0EFF")]
fn test_conn_handle_boundary_panic() {
    let _ = ConnHandle::new(0x0F00); // Just above the limit
}

#[test]
fn test_bluetooth_connection_state_comprehensive() {
    let mut connection_state = BluetoothConnectionState::default();

    // Test all connection state methods
    let mac_addr = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC];
    let device = BluetoothDeviceInfo::new(&mac_addr, b"Test Device").unwrap();

    connection_state.set_remote_device(device);
    connection_state.set_connected(true);
    connection_state.set_authenticated(true);
    connection_state.set_link_quality(255); // Maximum quality
    connection_state.set_remote_device_address([0xFF; 6]);
    connection_state.set_connection_handle(Some(ConnHandle::new(0x0EFF)));
    connection_state.set_link_type(0x02); // SCO
    connection_state.set_connection_phase(BluetoothConnectionPhase::Ready);

    // Verify all settings
    assert!(connection_state.is_connected());
    assert!(connection_state.is_authenticated());
    assert_eq!(connection_state.get_link_quality(), 255);
    assert_eq!(
        connection_state.get_remote_device_address(),
        Some([0xFF; 6])
    );
    assert_eq!(
        connection_state.get_connection_handle(),
        Some(ConnHandle::new(0x0EFF))
    );
    assert_eq!(connection_state.get_link_type(), 0x02);
    assert_eq!(
        connection_state.get_connection_phase(),
        BluetoothConnectionPhase::Ready
    );

    // Test clearing connection handle
    connection_state.set_connection_handle(None);
    assert_eq!(connection_state.get_connection_handle(), None);
}
