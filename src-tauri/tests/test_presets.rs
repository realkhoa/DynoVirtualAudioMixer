//! @file test_presets.rs
//! @brief Unit and integration tests for ProfilePreset serialization and ProfileManager disk operations.
//! @details Verifies JSON roundtrip serialization of routing graph presets, disk persistence,
//! directory scanning, profile deletion, special character sanitization, and corrupt file resilience.

use dyno_virtual_audio_mixer_lib::commands::{AudioGraphDto, ConnectionDto, NodeDto};
use dyno_virtual_audio_mixer_lib::presets::{ProfileManager, ProfilePreset};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// @brief Generates a unique temporary directory path for isolated preset unit testing.
/// @param test_name Name identifier of the specific test case.
/// @return PathBuf representing a unique temporary directory path.
fn create_test_dir(test_name: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time before epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("dyno_test_{test_name}_{timestamp}"));
    fs::create_dir_all(&dir).expect("Failed to create temporary test directory");
    dir
}

/// @brief Helper function to construct a sample AudioGraphDto with test nodes and wires.
/// @return Initialized AudioGraphDto populated with test nodes and connections.
fn create_sample_graph() -> AudioGraphDto {
    let node1 = NodeDto::new("node-mic", "Microphone", 0, 2);
    let node2 = NodeDto::new("node-master", "Master Output", 2, 0);
    let wire = ConnectionDto::new("wire-1", "node-mic", 0, "node-master", 0, 0.85);

    AudioGraphDto::new(vec![node1, node2], vec![wire])
}

/// @brief Tests JSON serialization and deserialization of ProfilePreset.
#[test]
fn test_profile_preset_serialization_deserialization() {
    let graph = create_sample_graph();
    let preset = ProfilePreset::new(
        "preset-alpha",
        "Streamer Balanced",
        "2026-11-03T01:30:00Z",
        graph,
    );

    let json_str = serde_json::to_string_pretty(&preset).expect("Serialization failed");
    let deserialized: ProfilePreset =
        serde_json::from_str(&json_str).expect("Deserialization failed");

    assert_eq!(preset.id, deserialized.id);
    assert_eq!(preset.name, deserialized.name);
    assert_eq!(preset.created_at, deserialized.created_at);
    assert_eq!(preset.graph.nodes.len(), deserialized.graph.nodes.len());
    assert_eq!(
        preset.graph.connections.len(),
        deserialized.graph.connections.len()
    );
    assert_eq!(preset, deserialized);
}

/// @brief Tests saving a profile to disk and loading it back cleanly.
#[test]
fn test_profile_manager_save_and_load() {
    let test_dir = create_test_dir("save_load");
    let manager = ProfileManager::new(test_dir.clone());

    let graph = create_sample_graph();
    let preset = ProfilePreset::new("preset-gaming", "Gaming Setup", "2026-11-03T01:00:00Z", graph);

    let saved_name = manager
        .save_profile(&preset)
        .expect("Failed to save profile preset");
    assert_eq!(saved_name, "Gaming Setup");

    // Load by friendly profile name
    let loaded = manager
        .load_profile("Gaming Setup")
        .expect("Failed to load profile by name");
    assert_eq!(loaded.id, "preset-gaming");
    assert_eq!(loaded.name, "Gaming Setup");
    assert_eq!(loaded.graph.nodes.len(), 2);

    // Load by profile id
    let loaded_by_id = manager
        .load_profile("preset-gaming")
        .expect("Failed to load profile by id");
    assert_eq!(loaded_by_id.name, "Gaming Setup");

    // Clean up
    let _ = fs::remove_dir_all(&test_dir);
}

/// @brief Tests listing all available profiles in the directory.
#[test]
fn test_profile_manager_list_profiles() {
    let test_dir = create_test_dir("list_profiles");
    let manager = ProfileManager::new(test_dir.clone());

    let p1 = ProfilePreset::new("p1", "Streaming Setup", "2026-11-03T01:00:00Z", create_sample_graph());
    let p2 = ProfilePreset::new("p2", "Gaming Rig", "2026-11-03T01:05:00Z", create_sample_graph());
    let p3 = ProfilePreset::new("p3", "Podcast Studio", "2026-11-03T01:10:00Z", create_sample_graph());

    manager.save_profile(&p1).unwrap();
    manager.save_profile(&p2).unwrap();
    manager.save_profile(&p3).unwrap();

    let list = manager.list_profiles().expect("Failed to list profiles");
    assert_eq!(list.len(), 3);
    assert_eq!(list, vec!["Gaming Rig", "Podcast Studio", "Streaming Setup"]);

    // Clean up
    let _ = fs::remove_dir_all(&test_dir);
}

/// @brief Tests deleting an existing profile and handling deletion of nonexistent profiles.
#[test]
fn test_profile_manager_delete_profile() {
    let test_dir = create_test_dir("delete_profile");
    let manager = ProfileManager::new(test_dir.clone());

    let preset = ProfilePreset::new("p-temp", "Temp Profile", "2026-11-03T01:00:00Z", create_sample_graph());
    manager.save_profile(&preset).unwrap();

    let deleted = manager
        .delete_profile("Temp Profile")
        .expect("Failed to delete existing profile");
    assert!(deleted);

    // Deleting again should return Ok(false)
    let deleted_again = manager
        .delete_profile("Temp Profile")
        .expect("Nonexistent delete should return Ok(false)");
    assert!(!deleted_again);

    // Loading should fail
    let load_res = manager.load_profile("Temp Profile");
    assert!(load_res.is_err());

    // Clean up
    let _ = fs::remove_dir_all(&test_dir);
}

/// @brief Tests handling of corrupt JSON files and nonexistent profiles.
#[test]
fn test_profile_manager_corrupt_and_invalid_file_handling() {
    let test_dir = create_test_dir("corrupt_handling");
    let manager = ProfileManager::new(test_dir.clone());

    // Write a corrupted file into the directory
    let corrupt_file = test_dir.join("corrupt_profile.json");
    fs::write(&corrupt_file, b"{\"id\": \"broken\", \"invalid\": [1, 2, }").unwrap();

    // Loading the corrupt profile should return Err
    let load_res = manager.load_profile("corrupt_profile");
    assert!(load_res.is_err());

    // Loading a completely nonexistent profile should return Err
    let nonexistent_res = manager.load_profile("completely_missing");
    assert!(nonexistent_res.is_err());

    // Save a valid profile alongside the corrupt one
    let valid = ProfilePreset::new("p-valid", "Valid Profile", "2026-11-03T01:00:00Z", create_sample_graph());
    manager.save_profile(&valid).unwrap();

    // Listing profiles should gracefully skip the corrupt profile and return the valid one
    let list = manager.list_profiles().expect("Listing should succeed despite corrupt files");
    assert_eq!(list, vec!["Valid Profile"]);

    // Clean up
    let _ = fs::remove_dir_all(&test_dir);
}

/// @brief Tests profile name sanitization when names contain illegal filesystem characters.
#[test]
fn test_profile_manager_sanitization_and_special_characters() {
    let test_dir = create_test_dir("sanitization");
    let manager = ProfileManager::new(test_dir.clone());

    let special_name = "Studio: 2026*Special? <Mix/Setup> | Final";
    let preset = ProfilePreset::new("p-special", special_name, "2026-11-03T01:00:00Z", create_sample_graph());

    let saved_name = manager
        .save_profile(&preset)
        .expect("Saving profile with special characters should succeed");
    assert_eq!(saved_name, special_name);

    let loaded = manager
        .load_profile(special_name)
        .expect("Loading profile by special character name should succeed");
    assert_eq!(loaded.name, special_name);
    assert_eq!(loaded.id, "p-special");

    // Clean up
    let _ = fs::remove_dir_all(&test_dir);
}
