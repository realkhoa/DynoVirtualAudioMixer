//! @file test_commands.rs
//! @brief Unit and integration tests for Tauri IPC commands, DTO serialization, and AppState concurrency.
//! @details Validates JSON serialization and deserialization for `AudioGraphDto`, `NodeDto`,
//! `ConnectionDto`, and `MeterUpdateDto`. Verifies thread-safe concurrent access to `AppState`
//! across reader and writer worker threads, and bidirectional conversion between DTOs and `RoutingMatrix`.

use dyno_virtual_audio_mixer_lib::audio::matrix::{MixerNode, RoutingMatrix};
use dyno_virtual_audio_mixer_lib::audio::types::MIN_PEAK_DB;
use dyno_virtual_audio_mixer_lib::audio::virtual_device::VirtualCableType;
use dyno_virtual_audio_mixer_lib::commands::{
    AppState, AudioGraphDto, ConnectionDto, MeterUpdateDto, NodeDto, NodeMeterDto,
};
use std::sync::atomic::Ordering;
use std::thread::{self, JoinHandle};

/// @brief Tests JSON serialization and deserialization for NodeDto and ConnectionDto.
/// @return Returns `()` on assertion success.
#[test]
fn test_node_and_connection_dto_serialization() {
    let node_dto = NodeDto {
        id: "node_spotify".to_string(),
        name: "Spotify Premium".to_string(),
        node_type: Some("app".to_string()),
        volume: 0.85,
        muted: false,
        input_channels: 0,
        output_channels: 2,
        is_monitor: false,
        mute_when_monitoring: false,
        is_offline: false,
        noise_gate_enabled: false,
        noise_gate_threshold_db: -45.0,
    };
    let node_json = serde_json::to_string(&node_dto).expect("NodeDto should serialize cleanly");
    assert!(node_json.contains("node_spotify"));
    assert!(node_json.contains("Spotify Premium"));
    assert!(node_json.contains("0.85"));

    let node_deser: NodeDto =
        serde_json::from_str(&node_json).expect("NodeDto should deserialize cleanly");
    assert_eq!(node_dto, node_deser);

    let connection_dto = ConnectionDto {
        id: "wire_01".to_string(),
        source_node_id: "node_spotify".to_string(),
        source_channel: 0,
        destination_node_id: "node_master".to_string(),
        destination_channel: 0,
        gain: 1.0,
        muted: false,
        phase_invert: false,
        is_monitor: false,
    };
    let conn_json =
        serde_json::to_string(&connection_dto).expect("ConnectionDto should serialize cleanly");
    assert!(conn_json.contains("wire_01"));
    assert!(conn_json.contains("node_spotify"));
    assert!(conn_json.contains("node_master"));

    let conn_deser: ConnectionDto =
        serde_json::from_str(&conn_json).expect("ConnectionDto should deserialize cleanly");
    assert_eq!(connection_dto, conn_deser);
}

/// @brief Tests JSON serialization and deserialization for AudioGraphDto and MeterUpdateDto.
/// @return Returns `()` on assertion success.
#[test]
fn test_graph_and_meter_dto_serialization() {
    let node_dto = NodeDto::new("node_mic", "Microphone", 1, 0);
    let conn_dto = ConnectionDto::new("wire_02", "node_mic", 0, "node_out", 0, 1.0);

    let graph_dto = AudioGraphDto::new(vec![node_dto], vec![conn_dto]);
    let graph_json =
        serde_json::to_string(&graph_dto).expect("AudioGraphDto should serialize cleanly");
    assert!(graph_json.contains("node_mic"));
    assert!(graph_json.contains("wire_02"));

    let graph_deser: AudioGraphDto =
        serde_json::from_str(&graph_json).expect("AudioGraphDto should deserialize cleanly");
    assert_eq!(graph_dto, graph_deser);

    let meter_node = NodeMeterDto {
        node_id: "node_mic".to_string(),
        peak_left: -12.5,
        peak_right: -14.2,
    };
    let meter_dto = MeterUpdateDto::new(1700000000000, vec![meter_node]);
    let meter_json =
        serde_json::to_string(&meter_dto).expect("MeterUpdateDto should serialize cleanly");
    assert!(meter_json.contains("node_mic"));
    assert!(meter_json.contains("-12.5"));
    assert!(meter_json.contains("-14.2"));

    let meter_deser: MeterUpdateDto =
        serde_json::from_str(&meter_json).expect("MeterUpdateDto should deserialize cleanly");
    assert_eq!(meter_dto, meter_deser);
}

/// @brief Helper to generate a test AudioGraphDto fixture.
/// @return Populated AudioGraphDto instance with two nodes and one wire.
fn create_test_graph_dto() -> AudioGraphDto {
    let node1 = NodeDto {
        id: "src_1".to_string(),
        name: "Microphone".to_string(),
        node_type: Some("input".to_string()),
        volume: 0.9,
        muted: false,
        input_channels: 1,
        output_channels: 0,
        is_monitor: false,
        mute_when_monitoring: false,
        is_offline: false,
        noise_gate_enabled: false,
        noise_gate_threshold_db: -45.0,
    };
    let node2 = NodeDto {
        id: "dst_1".to_string(),
        name: "Headphones".to_string(),
        node_type: Some("output".to_string()),
        volume: 1.0,
        muted: false,
        input_channels: 0,
        output_channels: 2,
        is_monitor: false,
        mute_when_monitoring: false,
        is_offline: false,
        noise_gate_enabled: false,
        noise_gate_threshold_db: -45.0,
    };
    let wire = ConnectionDto::new("wire_mic_to_hp", "src_1", 0, "dst_1", 0, 0.8);
    AudioGraphDto::new(vec![node1, node2], vec![wire])
}

/// @brief Tests converting from AudioGraphDto to RoutingMatrix.
/// @return Returns `()` on assertion success.
#[test]
fn test_dto_to_routing_matrix_conversion() {
    let original_graph = create_test_graph_dto();
    let matrix: RoutingMatrix = original_graph.to_routing_matrix();

    assert_eq!(matrix.nodes().len(), 2);
    assert_eq!(matrix.wires().len(), 1);

    let m_node1 = matrix.get_node("src_1").expect("src_1 node must exist");
    assert_eq!(m_node1.name, "Microphone");
    assert!((m_node1.volume - 0.9).abs() < f32::EPSILON);

    let m_wire = matrix.get_wire("wire_mic_to_hp").expect("wire must exist");
    assert_eq!(m_wire.source.node_id, "src_1");
    assert_eq!(m_wire.destination.node_id, "dst_1");
    assert!((m_wire.gain - 0.8).abs() < f32::EPSILON);
}

/// @brief Tests converting from RoutingMatrix back to AudioGraphDto.
/// @return Returns `()` on assertion success.
#[test]
fn test_routing_matrix_to_dto_conversion() {
    let original_graph = create_test_graph_dto();
    let matrix: RoutingMatrix = original_graph.to_routing_matrix();
    let roundtrip_graph = AudioGraphDto::from_routing_matrix(&matrix);

    assert_eq!(roundtrip_graph.nodes.len(), 2);
    assert_eq!(roundtrip_graph.connections.len(), 1);

    assert!(roundtrip_graph.nodes.iter().any(|n| n.id == "src_1"));
    assert!(roundtrip_graph.nodes.iter().any(|n| n.id == "dst_1"));
    assert!(roundtrip_graph
        .connections
        .iter()
        .any(|c| c.id == "wire_mic_to_hp"));
}

/// @brief Spawns concurrent reader threads inspecting matrix and virtual device states.
/// @param state Cloned application state handle.
/// @param count Number of reader threads to spawn.
/// @return Vector of thread join handles.
fn spawn_readers(state: &AppState, count: usize) -> Vec<JoinHandle<()>> {
    let mut handles = Vec::with_capacity(count);
    for _ in 0..count {
        let state_clone = state.clone();
        handles.push(thread::spawn(move || {
            for _ in 0..40 {
                let matrix = state_clone.read_matrix().unwrap();
                let _node_count = matrix.nodes().len();
                let vdevs = state_clone.read_virtual_devices().unwrap();
                let _dev_count = vdevs.list_devices().len();
                let _running = state_clone.is_running.load(Ordering::SeqCst);
                thread::yield_now();
            }
        }));
    }
    handles
}

/// @brief Spawns concurrent writer threads adding nodes to the routing matrix.
/// @param state Cloned application state handle.
/// @param count Number of writer threads to spawn.
/// @return Vector of thread join handles.
fn spawn_matrix_writers(state: &AppState, count: usize) -> Vec<JoinHandle<()>> {
    let mut handles = Vec::with_capacity(count);
    for thread_idx in 0..count {
        let state_clone = state.clone();
        handles.push(thread::spawn(move || {
            for i in 0..20 {
                let mut matrix = state_clone.write_matrix().unwrap();
                let node_id = format!("worker_{}_{}", thread_idx, i);
                matrix.add_node(MixerNode::new(&node_id, format!("Node {}", i), 2, 2));
                thread::yield_now();
            }
        }));
    }
    handles
}

/// @brief Spawns concurrent writer threads creating virtual devices.
/// @param state Cloned application state handle.
/// @param count Number of writer threads to spawn.
/// @return Vector of thread join handles.
fn spawn_vdev_writers(state: &AppState, count: usize) -> Vec<JoinHandle<()>> {
    let mut handles = Vec::with_capacity(count);
    for thread_idx in 0..count {
        let state_clone = state.clone();
        handles.push(thread::spawn(move || {
            for i in 0..10 {
                let mut vdevs = state_clone.write_virtual_devices().unwrap();
                let cable_name = format!("VDev-{}-{}", thread_idx, i);
                let _ = vdevs.create_virtual_device(
                    &cable_name,
                    VirtualCableType::SoftwareLoopbackEndpoint,
                );
                thread::yield_now();
            }
        }));
    }
    handles
}

/// @brief Tests AppState thread-safe concurrency across multiple reader and writer threads.
/// @return Returns `()` on assertion success.
#[test]
fn test_app_state_concurrency() {
    let state = AppState::new();
    assert!(state.is_running.load(Ordering::SeqCst));

    let mut handles = spawn_readers(&state, 4);
    handles.extend(spawn_matrix_writers(&state, 2));
    handles.extend(spawn_vdev_writers(&state, 2));

    for handle in handles {
        handle.join().expect("Worker thread panicked unexpectedly");
    }

    assert!(state.is_running.load(Ordering::SeqCst));
    let final_matrix = state.read_matrix().unwrap();
    assert!(final_matrix.nodes().len() >= 40);
    let final_vdevs = state.read_virtual_devices().unwrap();
    assert!(final_vdevs.list_devices().len() >= 20);
}

/// @brief Tests AppState meter snapshot generation logic.
/// @return Returns `()` on assertion success.
#[test]
fn test_app_state_meter_snapshot_generation() {
    let state = AppState::new();

    {
        let mut matrix = state.write_matrix().unwrap();
        matrix.add_node(MixerNode::new("app_spotify", "Spotify", 0, 2).with_volume(1.0));
        matrix.add_node(MixerNode::new("mic_in", "Microphone", 1, 0).with_muted(true));
    }

    let meter_snapshot = state.generate_meter_snapshot();
    assert_eq!(meter_snapshot.meters.len(), 2);

    let spotify_meter = meter_snapshot
        .meters
        .iter()
        .find(|m| m.node_id == "app_spotify")
        .expect("Spotify meter must exist");
    assert!(spotify_meter.peak_left >= MIN_PEAK_DB);
    assert!(spotify_meter.peak_right >= MIN_PEAK_DB);

    let mic_meter = meter_snapshot
        .meters
        .iter()
        .find(|m| m.node_id == "mic_in")
        .expect("Microphone meter must exist");
    assert!((mic_meter.peak_left - MIN_PEAK_DB).abs() < f32::EPSILON);
    assert!((mic_meter.peak_right - MIN_PEAK_DB).abs() < f32::EPSILON);
}
