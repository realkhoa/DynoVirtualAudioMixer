//! @file e2e_audio_smoke_test.rs
//! @brief End-to-end integration smoke test exercising the complete Dyno Virtual Audio Mixer pipeline.
//! @details Validates device discovery, virtual audio cable creation, pin-level routing,
//! lock-free ring buffer streaming, multi-stage audio mixing, soft-limiting, and preset persistence.

use dyno_virtual_audio_mixer_lib::audio::devices::list_audio_devices;
use dyno_virtual_audio_mixer_lib::audio::matrix::{
    AudioPinId, ConnectionWire, MixerNode, RoutingMatrix,
};
use dyno_virtual_audio_mixer_lib::audio::mixer::AudioMixerEngine;
use dyno_virtual_audio_mixer_lib::audio::process_scanner::list_audio_processes;
use dyno_virtual_audio_mixer_lib::audio::ring_buffer::AudioRingBuffer;
use dyno_virtual_audio_mixer_lib::audio::types::{calculate_peak_db, AudioChunk, MIN_PEAK_DB};
use dyno_virtual_audio_mixer_lib::audio::virtual_device::{
    VirtualCableType, VirtualDeviceDescriptor, VirtualDeviceManager,
};
use dyno_virtual_audio_mixer_lib::commands::{AppState, AudioGraphDto};
use dyno_virtual_audio_mixer_lib::presets::{ProfileManager, ProfilePreset};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// @brief Generates an isolated temporary directory for profile preset disk verification.
/// @param test_name Distinct name of the test scenario.
/// @return PathBuf pointing to the created temporary directory.
fn create_temp_e2e_dir(test_name: &str) -> PathBuf {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time before epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("dyno_e2e_{test_name}_{ts}"));
    fs::create_dir_all(&dir).expect("Failed to create temporary directory");
    dir
}

/// @brief Discovers system audio endpoints and creates a test virtual audio device.
/// @return Populated VirtualDeviceManager with created virtual endpoint.
fn verify_device_discovery() -> (VirtualDeviceManager, VirtualDeviceDescriptor) {
    let physical_devices = list_audio_devices().expect("Device enumeration failed");
    let _ = list_audio_processes().expect("Process scanning failed");
    println!("Queried {} audio devices", physical_devices.len());

    let mut vdev_mgr = VirtualDeviceManager::new();
    let drivers = vdev_mgr.detect_available_drivers();
    assert!(!drivers.is_empty(), "Must detect at least one audio driver");

    let descriptor = vdev_mgr
        .create_virtual_device("Dyno Broadcast Cable", VirtualCableType::SoftwareLoopbackEndpoint)
        .expect("Failed to create virtual audio device");

    assert_eq!(descriptor.name, "Dyno Broadcast Cable");
    assert!(descriptor.is_active);
    (vdev_mgr, descriptor)
}

/// @brief Builds a 3-node routing matrix linking App Source -> Virtual Bus -> Speakers.
/// @return Initialized and interconnected RoutingMatrix.
fn build_test_routing_matrix() -> RoutingMatrix {
    let mut matrix = RoutingMatrix::new();
    matrix.add_node(MixerNode::new("app_spotify", "Spotify Desktop", 0, 2).with_volume(1.0));
    matrix.add_node(MixerNode::new("vdev_cable", "Dyno Broadcast Cable", 2, 2).with_volume(1.0));
    matrix.add_node(MixerNode::new("out_speakers", "Realtek Audio", 2, 0).with_volume(0.8));

    let wires = vec![
        ConnectionWire::new("w1", AudioPinId::left("app_spotify"), AudioPinId::left("vdev_cable"), 1.0),
        ConnectionWire::new("w2", AudioPinId::right("app_spotify"), AudioPinId::right("vdev_cable"), 1.0),
        ConnectionWire::new("w3", AudioPinId::left("vdev_cable"), AudioPinId::left("out_speakers"), 1.0),
        ConnectionWire::new("w4", AudioPinId::right("vdev_cable"), AudioPinId::right("out_speakers"), 0.5),
    ];
    for wire in wires {
        matrix.connect(wire);
    }
    matrix
}

/// @brief Streams synthesized stereo audio through a lock-free ring buffer.
/// @return Rendered AudioChunk containing popped audio samples.
fn stream_through_ring_buffer() -> AudioChunk {
    let ring_buffer = AudioRingBuffer::new(2048);
    let (mut producer, mut consumer) = ring_buffer.split();

    let frames = 256usize;
    let mut raw_samples = Vec::with_capacity(frames * 2);
    for i in 0..frames {
        let t = (i as f32) / 48000.0;
        raw_samples.push((2.0 * std::f32::consts::PI * 440.0 * t).sin() * 0.6);
        raw_samples.push((2.0 * std::f32::consts::PI * 880.0 * t).sin() * 0.4);
    }

    assert_eq!(producer.push_samples(&raw_samples), raw_samples.len());
    let mut buffer = vec![0.0f32; raw_samples.len()];
    assert_eq!(consumer.pop_samples(&mut buffer), raw_samples.len());
    AudioChunk::new(buffer, 2, 48000)
}

/// @brief Verifies multi-stage audio mixing across Source, Virtual Bus, and Speakers.
/// @param matrix Routing matrix graph.
/// @param input_chunk Staged source audio chunk.
fn verify_multi_stage_mixing(matrix: &RoutingMatrix, input_chunk: &AudioChunk) {
    let mut engine = AudioMixerEngine::new(48000);
    *engine.matrix_mut() = matrix.clone();

    engine.submit_input("app_spotify", input_chunk.clone());
    let vdev_out = engine.process_destination("vdev_cable");
    assert_eq!(vdev_out.frame_count(), input_chunk.frame_count());

    engine.submit_input("vdev_cable", vdev_out);
    let speaker_out = engine.process_destination("out_speakers");
    assert_eq!(speaker_out.frame_count(), input_chunk.frame_count());

    for i in 0..input_chunk.frame_count() {
        let exp_l = input_chunk.samples[i * 2] * 1.0 * 0.8;
        let exp_r = input_chunk.samples[i * 2 + 1] * 0.5 * 0.8;
        assert!((speaker_out.samples[i * 2] - exp_l).abs() < 1e-4);
        assert!((speaker_out.samples[i * 2 + 1] - exp_r).abs() < 1e-4);
    }
}

/// @brief Tests soft limiter saturation and dBFS calculation algorithms.
fn verify_dsp_limiting_and_meters() {
    let mut matrix = RoutingMatrix::new();
    matrix.add_node(MixerNode::new("src", "Mic", 0, 2));
    matrix.add_node(MixerNode::new("dst", "Out", 2, 0));
    matrix.connect(ConnectionWire::new("w1", AudioPinId::left("src"), AudioPinId::left("dst"), 2.0));
    matrix.set_soft_limiter(true);

    let mut inputs = HashMap::new();
    inputs.insert("src".to_string(), AudioChunk::new(vec![2.5, 0.0], 2, 48000));
    let out = matrix.process_frame(&inputs, "dst");
    assert!((out.samples[0] - (5.0f32).tanh()).abs() < 1e-4);
    assert!(out.samples[0] < 1.0);

    let (db_l, _) = calculate_peak_db(&[0.5, 0.0], 2);
    assert!((db_l - (-6.0206)).abs() < 0.02);

    let (silence_db, _) = AudioChunk::silence(32, 2, 48000).calculate_peak_db();
    assert_eq!(silence_db, MIN_PEAK_DB);
}

/// @brief Verifies AppState telemetry snapshot and muting response.
/// @param matrix Routing matrix to test.
/// @param vdev_mgr Virtual device manager instance.
fn verify_app_state_telemetry(matrix: &RoutingMatrix, vdev_mgr: VirtualDeviceManager) {
    let state = AppState::with_matrix_and_devices(matrix.clone(), vdev_mgr);
    let snapshot = state.generate_meter_snapshot();
    assert_eq!(snapshot.meters.len(), 3);

    let mut guard = state.write_matrix().expect("Failed to lock matrix");
    if let Some(node) = guard.get_node_mut("app_spotify") {
        node.muted = true;
    }
    drop(guard);

    let muted_snapshot = state.generate_meter_snapshot();
    let meter = muted_snapshot.meters.iter().find(|m| m.node_id == "app_spotify").unwrap();
    assert_eq!(meter.peak_left, MIN_PEAK_DB);
}

/// @brief Verifies preset disk persistence, directory scanning, and graph reload.
/// @param matrix Routing matrix to persist and reload.
fn verify_preset_persistence(matrix: &RoutingMatrix) {
    let temp_dir = create_temp_e2e_dir("preset_e2e");
    let mgr = ProfileManager::new(temp_dir.clone());
    let graph = AudioGraphDto::from_routing_matrix(matrix);
    let preset = ProfilePreset::new("p1", "Studio Broadcast", "2026-11-03T01:00:00Z", graph.clone());

    assert_eq!(mgr.save_profile(&preset).expect("Save failed"), "Studio Broadcast");
    let list = mgr.list_profiles().expect("List failed");
    assert_eq!(list, vec!["Studio Broadcast".to_string()]);

    let loaded = mgr.load_profile("Studio Broadcast").expect("Load failed");
    assert_eq!(loaded.graph, graph);
    let reloaded_matrix = loaded.graph.to_routing_matrix();
    assert_eq!(reloaded_matrix.nodes().len(), 3);
    assert_eq!(reloaded_matrix.wires().len(), 4);
    let _ = fs::remove_dir_all(&temp_dir);
}

/// @brief Executes complete end-to-end integration smoke test across all audio subsystems.
/// @return Returns `()` on assertion success.
#[test]
fn test_e2e_full_audio_pipeline_smoke() {
    let (vdev_mgr, _) = verify_device_discovery();
    let matrix = build_test_routing_matrix();
    let input_chunk = stream_through_ring_buffer();

    verify_multi_stage_mixing(&matrix, &input_chunk);
    verify_dsp_limiting_and_meters();
    verify_app_state_telemetry(&matrix, vdev_mgr);
    verify_preset_persistence(&matrix);
}
