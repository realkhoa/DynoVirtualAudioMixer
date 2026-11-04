//! @file test_virtual_device.rs
//! @brief Unit and integration tests for VirtualDeviceManager, VirtualCableType, and VirtualDeviceDescriptor.
//! @details Verifies driver detection (VB-Cable, Virtual Audio Cable, SoftwareLoopback fallback),
//! virtual device creation, unique identification, duplicate naming resolution, endpoint binding,
//! device removal, and JSON serialization for Tauri IPC.

use dyno_virtual_audio_mixer_lib::audio::virtual_device::{
    get_driver_cache_dir, is_windows_cable_installed, VirtualCableType, VirtualDeviceDescriptor,
    VirtualDeviceManager,
};

/// @brief Test 1: VirtualDeviceManager instantiation and driver detection returns known types or fallback software loopback.
#[test]
fn test_virtual_device_manager_detection() {
    let manager = VirtualDeviceManager::new();
    let drivers = manager.detect_available_drivers();
    // Driver detection must always report at least SoftwareLoopbackEndpoint if no hardware drivers exist
    assert!(!drivers.is_empty(), "Drivers list should not be empty");
    assert!(
        drivers.contains(&VirtualCableType::SoftwareLoopbackEndpoint),
        "SoftwareLoopbackEndpoint should always be available as a fallback"
    );
}

/// @brief Test 2: Creating a named virtual device registers descriptor with unique ID and active state.
#[test]
fn test_create_virtual_device() {
    let mut manager = VirtualDeviceManager::new();
    let result = manager.create_virtual_device("Game Audio", VirtualCableType::SoftwareLoopbackEndpoint);
    assert!(result.is_ok(), "Creating a virtual device should succeed");

    let device = result.unwrap();
    assert_eq!(device.name, "Game Audio");
    assert_eq!(device.driver_type, VirtualCableType::SoftwareLoopbackEndpoint);
    assert!(device.is_active, "Newly created device should be active");
    assert!(!device.id.is_empty(), "Device should have a non-empty unique ID");
    assert_eq!(device.channels, 2);
    assert_eq!(device.sample_rate, 48000);

    let listed = manager.list_devices();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, device.id);

    let fetched = manager.get_device(&device.id);
    assert!(fetched.is_some());
    assert_eq!(fetched.unwrap().name, "Game Audio");
}

/// @brief Test 3: Duplicate device name handling or auto-incrementing naming.
#[test]
fn test_duplicate_device_naming() {
    let mut manager = VirtualDeviceManager::new();
    let dev1 = manager
        .create_virtual_device("Discord Aux", VirtualCableType::SoftwareLoopbackEndpoint)
        .expect("First device creation should succeed");
    let dev2 = manager
        .create_virtual_device("Discord Aux", VirtualCableType::SoftwareLoopbackEndpoint)
        .expect("Second device creation with duplicate name should handle collisions");

    assert_ne!(dev1.id, dev2.id, "IDs must be unique even with identical requested names");
    assert!(
        dev2.name == "Discord Aux (2)" || (dev2.name == "Discord Aux" && dev1.id != dev2.id),
        "Duplicate device must be cleanly disambiguated or distinguished by ID"
    );
    assert_eq!(manager.list_devices().len(), 2);
}

/// @brief Test 4: Removing virtual device by ID.
#[test]
fn test_remove_virtual_device() {
    let mut manager = VirtualDeviceManager::new();
    let dev = manager
        .create_virtual_device("Music Stream", VirtualCableType::SoftwareLoopbackEndpoint)
        .expect("Device creation should succeed");

    assert!(manager.get_device(&dev.id).is_some());

    let removed = manager.remove_virtual_device(&dev.id);
    assert_eq!(removed, Ok(true));
    assert!(manager.get_device(&dev.id).is_none());
    assert_eq!(manager.list_devices().len(), 0);

    // Removing a non-existent device returns Ok(false)
    let remove_again = manager.remove_virtual_device(&dev.id);
    assert_eq!(remove_again, Ok(false));
}

/// @brief Test 5: Serialization of VirtualDeviceDescriptor to JSON for Tauri IPC.
#[test]
fn test_virtual_device_descriptor_serialization() {
    let descriptor = VirtualDeviceDescriptor {
        id: "vdev-test-123".to_string(),
        name: "Virtual Cable A".to_string(),
        driver_type: VirtualCableType::VBCable,
        render_endpoint_id: Some("out-vbcable-render".to_string()),
        capture_endpoint_id: Some("in-vbcable-capture".to_string()),
        channels: 2,
        sample_rate: 48000,
        is_active: true,
    };

    let serialized = serde_json::to_string(&descriptor)
        .expect("VirtualDeviceDescriptor should serialize cleanly to JSON string");
    assert!(serialized.contains("Virtual Cable A"));
    assert!(serialized.contains("\"driver_type\":\"VBCable\""));
    assert!(serialized.contains("\"is_active\":true"));
    assert!(serialized.contains("out-vbcable-render"));

    let deserialized: VirtualDeviceDescriptor = serde_json::from_str(&serialized)
        .expect("VirtualDeviceDescriptor should deserialize cleanly from JSON string");
    assert_eq!(descriptor, deserialized);
    assert_eq!(deserialized.channels, 2);
    assert_eq!(deserialized.sample_rate, 48000);
    assert!(deserialized.is_active);
}

/// @brief Test 6: Empty device name error validation.
#[test]
fn test_create_virtual_device_empty_name() {
    let mut manager = VirtualDeviceManager::new();
    let result = manager.create_virtual_device("   ", VirtualCableType::SoftwareLoopbackEndpoint);
    assert!(result.is_err(), "Empty or whitespace-only name should fail");
}

/// @brief Test 7: Toggling device active status.
#[test]
fn test_set_device_active() {
    let mut manager = VirtualDeviceManager::new();
    let dev = manager
        .create_virtual_device("Monitor Bus", VirtualCableType::SoftwareLoopbackEndpoint)
        .expect("Device creation should succeed");

    assert!(dev.is_active);
    let updated = manager.set_device_active(&dev.id, false);
    assert_eq!(updated, Ok(true));
    assert!(!manager.get_device(&dev.id).unwrap().is_active);

    let updated_back = manager.set_device_active(&dev.id, true);
    assert_eq!(updated_back, Ok(true));
    assert!(manager.get_device(&dev.id).unwrap().is_active);

    let not_found = manager.set_device_active("non-existent-id", false);
    assert_eq!(not_found, Ok(false));
}

/// @brief Test 8: Multi-channel (4, 8, 16 ch) virtual device creation and channel verification.
/// @return Returns `()` on assertion success.
#[test]
fn test_multichannel_virtual_device_creation() {
    let mut manager = VirtualDeviceManager::new();
    let desc_4ch = manager
        .create_virtual_device_with_channels(
            "Podcast Quad Out",
            VirtualCableType::SoftwareLoopbackEndpoint,
            4,
        )
        .expect("4-channel virtual device should be created successfully");
    assert_eq!(desc_4ch.channels, 4);
    assert_eq!(desc_4ch.name, "Podcast Quad Out");

    let desc_8ch = manager
        .create_virtual_device_with_channels(
            "Studio MultiTrack",
            VirtualCableType::SoftwareLoopbackEndpoint,
            8,
        )
        .expect("8-channel virtual device should be created successfully");
    assert_eq!(desc_8ch.channels, 8);
    assert_eq!(desc_8ch.name, "Studio MultiTrack");

    let desc_16ch = manager
        .create_virtual_device_with_channels(
            "Pro Studio 16ch",
            VirtualCableType::SoftwareLoopbackEndpoint,
            16,
        )
        .expect("16-channel virtual device should be created successfully");
    assert_eq!(desc_16ch.channels, 16);
}

/// @brief Test 9: Resolution of driver cache directory structure.
#[test]
fn test_driver_cache_dir_resolution() {
    let cache_dir = get_driver_cache_dir();
    let path_str = cache_dir.to_string_lossy();
    assert!(
        path_str.contains("DynoVirtualAudioMixer") && path_str.contains("driver_cache"),
        "Driver cache directory must contain expected path components"
    );
}

/// @brief Test 10: Non-panicking query of Windows audio cable driver installation status.
#[test]
fn test_is_windows_cable_installed_execution() {
    // Calling is_windows_cable_installed should execute cleanly without panicking
    let _status = is_windows_cable_installed();
}
