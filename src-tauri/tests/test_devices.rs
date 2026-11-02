//! @file test_devices.rs
//! @brief Integration and unit tests for audio device enumeration and process audio detection.
//! @details Verifies serialization, deserialization, type classification, device filtering,
//! real or headless hardware query behavior, and audio session process scanning.

use dyno_virtual_audio_mixer_lib::audio::devices::{
    list_audio_devices, AudioDeviceDescriptor, AudioDeviceType,
};
use dyno_virtual_audio_mixer_lib::audio::process_scanner::{
    list_audio_processes, AudioProcessDescriptor,
};

/// @brief Tests JSON serialization and deserialization roundtrip for AudioDeviceDescriptor.
#[test]
fn test_audio_device_descriptor_serialization() {
    let descriptor = AudioDeviceDescriptor {
        id: "endpoint-wasapi-render-01".to_string(),
        name: "Speakers (Realtek High Definition Audio)".to_string(),
        device_type: AudioDeviceType::Output,
        channels: 2,
        sample_rate: 48000,
        is_default: true,
    };

    let serialized = serde_json::to_string(&descriptor)
        .expect("AudioDeviceDescriptor should serialize cleanly to JSON string");
    assert!(serialized.contains("Speakers (Realtek High Definition Audio)"));
    assert!(serialized.contains("\"device_type\":\"Output\""));
    assert!(serialized.contains("\"is_default\":true"));

    let deserialized: AudioDeviceDescriptor = serde_json::from_str(&serialized)
        .expect("AudioDeviceDescriptor should deserialize cleanly from JSON string");
    assert_eq!(descriptor, deserialized);
    assert_eq!(deserialized.channels, 2);
    assert_eq!(deserialized.sample_rate, 48000);
    assert!(deserialized.is_default);
    assert!(deserialized.device_type.is_output());
    assert!(!deserialized.device_type.is_input());
}

/// @brief Tests JSON serialization and deserialization roundtrip for AudioProcessDescriptor.
#[test]
fn test_audio_process_descriptor_serialization() {
    let process = AudioProcessDescriptor {
        pid: 12345,
        name: "spotify.exe".to_string(),
        title: "Spotify Free".to_string(),
        has_audio_session: true,
    };

    let serialized = serde_json::to_string(&process)
        .expect("AudioProcessDescriptor should serialize cleanly to JSON string");
    assert!(serialized.contains("spotify.exe"));
    assert!(serialized.contains("\"has_audio_session\":true"));

    let deserialized: AudioProcessDescriptor = serde_json::from_str(&serialized)
        .expect("AudioProcessDescriptor should deserialize cleanly from JSON string");
    assert_eq!(process, deserialized);
    assert_eq!(deserialized.pid, 12345);
    assert_eq!(deserialized.name, "spotify.exe");
    assert_eq!(deserialized.title, "Spotify Free");
    assert!(deserialized.has_audio_session);
}

/// @brief Tests AudioDeviceType classification helpers and string conversions.
#[test]
fn test_audio_device_type_helpers() {
    let input = AudioDeviceType::Input;
    let output = AudioDeviceType::Output;
    let virtual_dev = AudioDeviceType::Virtual;

    assert!(input.is_input());
    assert!(!input.is_output());
    assert!(!input.is_virtual());

    assert!(!output.is_input());
    assert!(output.is_output());
    assert!(!output.is_virtual());

    assert!(!virtual_dev.is_input());
    assert!(!virtual_dev.is_output());
    assert!(virtual_dev.is_virtual());
}

/// @brief Tests filtering device collections by type (Input vs Output vs Virtual).
#[test]
fn test_filter_audio_devices_by_type() {
    let devices = [
        AudioDeviceDescriptor {
            id: "dev-in-1".to_string(),
            name: "Microphone Array".to_string(),
            device_type: AudioDeviceType::Input,
            channels: 2,
            sample_rate: 48000,
            is_default: true,
        },
        AudioDeviceDescriptor {
            id: "dev-out-1".to_string(),
            name: "Headphones".to_string(),
            device_type: AudioDeviceType::Output,
            channels: 2,
            sample_rate: 48000,
            is_default: true,
        },
        AudioDeviceDescriptor {
            id: "dev-virt-1".to_string(),
            name: "Dyno Virtual Cable".to_string(),
            device_type: AudioDeviceType::Virtual,
            channels: 2,
            sample_rate: 48000,
            is_default: false,
        },
    ];

    let inputs: Vec<_> = devices.iter().filter(|d| d.device_type.is_input()).collect();
    let outputs: Vec<_> = devices.iter().filter(|d| d.device_type.is_output()).collect();
    let virtuals: Vec<_> = devices.iter().filter(|d| d.device_type.is_virtual()).collect();

    assert_eq!(inputs.len(), 1);
    assert_eq!(inputs[0].name, "Microphone Array");

    assert_eq!(outputs.len(), 1);
    assert_eq!(outputs[0].name, "Headphones");

    assert_eq!(virtuals.len(), 1);
    assert_eq!(virtuals[0].name, "Dyno Virtual Cable");
}

/// @brief Tests live audio device enumeration on host machine without panics.
/// @details Verifies that list_audio_devices() returns Ok(Vec) and descriptors have non-empty
/// identifiers and valid channel / sample rate values. In a desktop environment with audio,
/// at least one output device is typically detected; in headless environments, an empty list
/// is handled gracefully without errors.
#[test]
fn test_list_audio_devices_live() {
    let result = list_audio_devices();
    assert!(result.is_ok(), "list_audio_devices should not return an error: {:?}", result.err());

    let devices = result.unwrap();
    println!("Enumerated {} audio devices on host system:", devices.len());
    for dev in &devices {
        println!(
            " - [{:?}] {} (id={}, channels={}, rate={}Hz, default={})",
            dev.device_type, dev.name, dev.id, dev.channels, dev.sample_rate, dev.is_default
        );
        assert!(!dev.id.is_empty(), "Device id must not be empty");
        assert!(!dev.name.is_empty(), "Device name must not be empty");
        assert!(dev.channels > 0, "Device channels must be positive");
        assert!(dev.sample_rate > 0, "Device sample rate must be positive");
    }
}

/// @brief Tests live or system audio process enumeration without panics.
/// @details Verifies that list_audio_processes() returns Ok(Vec), and any discovered
/// processes have valid process IDs and non-empty executable names.
#[test]
fn test_list_audio_processes_live() {
    let result = list_audio_processes();
    assert!(result.is_ok(), "list_audio_processes should return Ok: {:?}", result.err());

    let processes = result.unwrap();
    println!("Discovered {} audio-related processes:", processes.len());
    for proc in &processes {
        println!(
            " - PID {}: '{}' (Title: '{}', HasAudioSession={})",
            proc.pid, proc.name, proc.title, proc.has_audio_session
        );
        assert!(proc.pid > 0 || proc.name == "System", "PID must be valid");
        assert!(!proc.name.is_empty(), "Process executable name must not be empty");
    }
}
