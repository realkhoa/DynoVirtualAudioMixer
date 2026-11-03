//! @file test_matrix.rs
//! @brief Integration and unit tests for audio routing matrix and real-time mixer pipeline.
//! @details Validates pin-level wiring connections, 1-to-1 direct routing, stereo downmixing,
//! mono splitting to stereo, node and wire muting, multi-source summing with smooth hyperbolic
//! tangent (tanh) soft saturation, and AudioMixerEngine frame processing.

use dyno_virtual_audio_mixer_lib::audio::matrix::{
    AudioPinId, ConnectionWire, MixerNode, RoutingMatrix,
};
use dyno_virtual_audio_mixer_lib::audio::mixer::AudioMixerEngine;
use dyno_virtual_audio_mixer_lib::audio::types::AudioChunk;
use std::collections::HashMap;

/// @brief Tests serialization and deserialization roundtrip for matrix nodes and connection wires.
#[test]
fn test_matrix_primitives_serialization() {
    let pin_src = AudioPinId::new("mic_node", 0);
    let pin_dst = AudioPinId::new("master_out", 1);
    let wire = ConnectionWire::new("wire_01", pin_src.clone(), pin_dst.clone(), 0.85);

    let wire_json = serde_json::to_string(&wire).expect("ConnectionWire should serialize cleanly");
    assert!(wire_json.contains("mic_node"));
    assert!(wire_json.contains("master_out"));
    assert!(wire_json.contains("0.85"));

    let wire_deser: ConnectionWire =
        serde_json::from_str(&wire_json).expect("ConnectionWire should deserialize cleanly");
    assert_eq!(wire, wire_deser);

    let node = MixerNode::new("app_spotify", "Spotify Free", 0, 2)
        .with_volume(0.75)
        .with_muted(false);

    let node_json = serde_json::to_string(&node).expect("MixerNode should serialize cleanly");
    assert!(node_json.contains("Spotify Free"));
    let node_deser: MixerNode =
        serde_json::from_str(&node_json).expect("MixerNode should deserialize cleanly");
    assert_eq!(node, node_deser);
}

/// @brief Test 1: Direct 1-to-1 routing (Source Node L/R -> Dest Node L/R).
/// @details Verifies exact sample propagation and volume gain.
#[test]
fn test_direct_one_to_one_routing() {
    let mut matrix = RoutingMatrix::new();

    let src = MixerNode::new("src_stereo", "Source Stereo", 0, 2).with_volume(1.0);
    let dst = MixerNode::new("dst_stereo", "Destination Stereo", 2, 2).with_volume(0.5);
    matrix.add_node(src);
    matrix.add_node(dst);

    // Connect L -> L and R -> R with wire gain 1.0
    matrix.connect(ConnectionWire::new(
        "wire_l",
        AudioPinId::new("src_stereo", 0),
        AudioPinId::new("dst_stereo", 0),
        1.0,
    ));
    matrix.connect(ConnectionWire::new(
        "wire_r",
        AudioPinId::new("src_stereo", 1),
        AudioPinId::new("dst_stereo", 1),
        1.0,
    ));

    // Provide test interleaved stereo samples: frame 0: [0.4, 0.8], frame 1: [0.2, -0.6]
    let input_chunk = AudioChunk::new(vec![0.4, 0.8, 0.2, -0.6], 2, 48000);
    let mut inputs = HashMap::new();
    inputs.insert("src_stereo".to_string(), input_chunk);

    let output = matrix.process_frame(&inputs, "dst_stereo");

    assert_eq!(output.channels, 2);
    assert_eq!(output.sample_rate, 48000);
    assert_eq!(output.frame_count(), 2);

    // Dest volume is 0.5, wire gain is 1.0 -> sample * 1.0 * 0.5
    // Frame 0
    assert!((output.samples[0] - 0.20).abs() < 1e-5, "Expected 0.20, got {}", output.samples[0]);
    assert!((output.samples[1] - 0.40).abs() < 1e-5, "Expected 0.40, got {}", output.samples[1]);
    // Frame 1
    assert!((output.samples[2] - 0.10).abs() < 1e-5, "Expected 0.10, got {}", output.samples[2]);
    assert!((output.samples[3] - (-0.30)).abs() < 1e-5, "Expected -0.30, got {}", output.samples[3]);
}

/// @brief Test 2: Stereo downmixing to mono (Source L/R -> Dest Mono) verifies (L + R) * 0.5.
#[test]
fn test_stereo_downmix_to_mono() {
    let mut matrix = RoutingMatrix::new();

    let src = MixerNode::new("src_stereo", "Source Stereo", 0, 2);
    let dst = MixerNode::new("dst_mono", "Destination Mono", 1, 1).with_volume(1.0);
    matrix.add_node(src);
    matrix.add_node(dst);

    // Connect L (ch 0) -> Mono (ch 0) with gain 0.5
    matrix.connect(ConnectionWire::new(
        "wire_downmix_l",
        AudioPinId::new("src_stereo", 0),
        AudioPinId::new("dst_mono", 0),
        0.5,
    ));
    // Connect R (ch 1) -> Mono (ch 0) with gain 0.5
    matrix.connect(ConnectionWire::new(
        "wire_downmix_r",
        AudioPinId::new("src_stereo", 1),
        AudioPinId::new("dst_mono", 0),
        0.5,
    ));

    // Input: L = 0.6, R = 0.4
    let input_chunk = AudioChunk::new(vec![0.6, 0.4], 2, 48000);
    let mut inputs = HashMap::new();
    inputs.insert("src_stereo".to_string(), input_chunk);

    let output = matrix.process_frame(&inputs, "dst_mono");

    assert_eq!(output.channels, 1);
    assert_eq!(output.frame_count(), 1);
    // (0.6 * 0.5) + (0.4 * 0.5) = 0.3 + 0.2 = 0.5
    assert!((output.samples[0] - 0.5).abs() < 1e-5, "Expected 0.5, got {}", output.samples[0]);
}

/// @brief Test 3: Mono channel split to stereo (Source Ch 0 -> Dest L and Dest R) verifies cloned channels.
#[test]
fn test_mono_split_to_stereo() {
    let mut matrix = RoutingMatrix::new();

    let src = MixerNode::new("src_mono", "Microphone Mono", 0, 1);
    let dst = MixerNode::new("dst_stereo", "Headphones Stereo", 2, 2);
    matrix.add_node(src);
    matrix.add_node(dst);

    // Connect Mono ch 0 to Dest Left (ch 0) and Dest Right (ch 1)
    matrix.connect(ConnectionWire::new(
        "wire_split_l",
        AudioPinId::new("src_mono", 0),
        AudioPinId::new("dst_stereo", 0),
        1.0,
    ));
    matrix.connect(ConnectionWire::new(
        "wire_split_r",
        AudioPinId::new("src_mono", 0),
        AudioPinId::new("dst_stereo", 1),
        1.0,
    ));

    let input_chunk = AudioChunk::new(vec![0.75, -0.45], 1, 48000);
    let mut inputs = HashMap::new();
    inputs.insert("src_mono".to_string(), input_chunk);

    let output = matrix.process_frame(&inputs, "dst_stereo");

    assert_eq!(output.channels, 2);
    assert_eq!(output.frame_count(), 2);
    // Frame 0: both L and R should be 0.75
    assert!((output.samples[0] - 0.75).abs() < 1e-5);
    assert!((output.samples[1] - 0.75).abs() < 1e-5);
    // Frame 1: both L and R should be -0.45
    assert!((output.samples[2] - (-0.45)).abs() < 1e-5);
    assert!((output.samples[3] - (-0.45)).abs() < 1e-5);
}

/// @brief Test 4a: Mute verification for connection wires.
#[test]
fn test_mute_wire_outputs_silence() {
    let mut matrix = RoutingMatrix::new();
    matrix.add_node(MixerNode::new("src", "Source", 0, 1));
    matrix.add_node(MixerNode::new("dst", "Destination", 1, 1));

    let wire = ConnectionWire::new("w", AudioPinId::new("src", 0), AudioPinId::new("dst", 0), 1.0).muted(true);
    matrix.connect(wire);

    let mut inputs = HashMap::new();
    inputs.insert("src".to_string(), AudioChunk::new(vec![0.88, 0.99], 1, 48000));

    let output = matrix.process_frame(&inputs, "dst");
    assert!(output.samples.iter().all(|&s| s == 0.0));
}

/// @brief Test 4b: Mute verification for source and destination nodes.
#[test]
fn test_mute_nodes_outputs_silence() {
    let mut matrix = RoutingMatrix::new();
    matrix.add_node(MixerNode::new("src", "Source", 0, 1));
    matrix.add_node(MixerNode::new("dst", "Destination", 1, 1));

    matrix.connect(ConnectionWire::new("w", AudioPinId::new("src", 0), AudioPinId::new("dst", 0), 1.0));
    let mut inputs = HashMap::new();
    inputs.insert("src".to_string(), AudioChunk::new(vec![0.88, 0.99], 1, 48000));

    // Mute source node
    matrix.get_node_mut("src").unwrap().muted = true;
    let output_src_muted = matrix.process_frame(&inputs, "dst");
    assert!(output_src_muted.samples.iter().all(|&s| s == 0.0));

    // Unmute source node, mute destination node
    matrix.get_node_mut("src").unwrap().muted = false;
    matrix.get_node_mut("dst").unwrap().muted = true;
    let output_dst_muted = matrix.process_frame(&inputs, "dst");
    assert!(output_dst_muted.samples.iter().all(|&s| s == 0.0));
}

/// @brief Test 5: Summing multiple sources with soft limiter prevents harsh digital clipping.
/// @details Confirms that signals with amplitude exceeding 1.0 are smoothly clamped via tanh.
#[test]
fn test_multi_source_summing_with_soft_limiter() {
    let mut matrix = RoutingMatrix::new();

    let src1 = MixerNode::new("src1", "Source 1", 0, 1);
    let src2 = MixerNode::new("src2", "Source 2", 0, 1);
    let dst = MixerNode::new("dst", "Destination Sum", 1, 1);
    matrix.add_node(src1);
    matrix.add_node(src2);
    matrix.add_node(dst);

    matrix.connect(ConnectionWire::new(
        "w1",
        AudioPinId::new("src1", 0),
        AudioPinId::new("dst", 0),
        1.0,
    ));
    matrix.connect(ConnectionWire::new(
        "w2",
        AudioPinId::new("src2", 0),
        AudioPinId::new("dst", 0),
        1.0,
    ));

    // Source 1 has sample 0.8, Source 2 has sample 0.7 -> sum before limiter = 1.5
    let mut inputs = HashMap::new();
    inputs.insert("src1".to_string(), AudioChunk::new(vec![0.8], 1, 48000));
    inputs.insert("src2".to_string(), AudioChunk::new(vec![0.7], 1, 48000));

    let output = matrix.process_frame(&inputs, "dst");

    assert_eq!(output.frame_count(), 1);
    // Raw sum would be 1.5 (harsh digital clipping > 1.0).
    // With soft limiting, 1.5 is clamped smoothly via tanh(1.5).
    let expected = 1.5f32.tanh();
    assert!(
        (output.samples[0] - expected).abs() < 1e-4,
        "Expected tanh(1.5) = {}, got {}",
        expected,
        output.samples[0]
    );
    assert!(output.samples[0] < 1.0, "Sample must be safely below 1.0 full scale");
}

/// @brief Tests node removal and wire disconnection lifecycle in RoutingMatrix.
#[test]
fn test_matrix_topology_modifications() {
    let mut matrix = RoutingMatrix::new();
    matrix.add_node(MixerNode::new("n1", "Node 1", 2, 2));
    matrix.add_node(MixerNode::new("n2", "Node 2", 2, 2));

    matrix.connect(ConnectionWire::new(
        "w1",
        AudioPinId::new("n1", 0),
        AudioPinId::new("n2", 0),
        1.0,
    ));

    assert_eq!(matrix.nodes().len(), 2);
    assert_eq!(matrix.wires().len(), 1);

    // Disconnect wire
    let removed_wire = matrix.disconnect("w1");
    assert!(removed_wire.is_some());
    assert_eq!(matrix.wires().len(), 0);

    // Reconnect and remove node -> should cascade remove connected wires
    matrix.connect(ConnectionWire::new(
        "w2",
        AudioPinId::new("n1", 0),
        AudioPinId::new("n2", 0),
        1.0,
    ));
    assert_eq!(matrix.wires().len(), 1);
    let removed_node = matrix.remove_node("n1");
    assert!(removed_node.is_some());
    assert_eq!(matrix.nodes().len(), 1);
    assert_eq!(matrix.wires().len(), 0, "Wires connected to removed node should be cleared");
}

/// @brief Tests AudioMixerEngine input ingestion and destination frame processing.
#[test]
fn test_audio_mixer_engine_pipeline() {
    let mut engine = AudioMixerEngine::new(48000);

    let src = MixerNode::new("app_browser", "Browser", 0, 2);
    let dst = MixerNode::new("master_out", "Speakers", 2, 2);
    engine.add_node(src);
    engine.add_node(dst);

    engine.connect(ConnectionWire::new(
        "w_left",
        AudioPinId::new("app_browser", 0),
        AudioPinId::new("master_out", 0),
        1.0,
    ));
    engine.connect(ConnectionWire::new(
        "w_right",
        AudioPinId::new("app_browser", 1),
        AudioPinId::new("master_out", 1),
        1.0,
    ));

    // Ingest audio chunk
    let input_chunk = AudioChunk::new(vec![0.3, 0.6, 0.4, 0.8], 2, 48000);
    engine.submit_input("app_browser", input_chunk);

    let rendered = engine.process_destination("master_out");
    assert_eq!(rendered.channels, 2);
    assert_eq!(rendered.frame_count(), 2);
    assert!((rendered.samples[0] - 0.3).abs() < 1e-5);
    assert!((rendered.samples[1] - 0.6).abs() < 1e-5);
    assert!((rendered.samples[2] - 0.4).abs() < 1e-5);
    assert!((rendered.samples[3] - 0.8).abs() < 1e-5);
}

/// @brief Tests wire gain attenuation and 180-degree phase inversion (`phase_invert`).
/// @return Returns `()` on assertion success.
#[test]
fn test_wire_phase_inversion_and_attenuation() {
    let mut matrix = RoutingMatrix::new();
    let src = MixerNode::new("src_tone", "Tone Generator", 0, 1).with_volume(1.0);
    let dst = MixerNode::new("dst_out", "Output", 1, 1).with_volume(1.0);
    matrix.add_node(src);
    matrix.add_node(dst);

    let mut wire = ConnectionWire::new(
        "wire_inverted",
        AudioPinId::new("src_tone", 0),
        AudioPinId::new("dst_out", 0),
        0.5,
    );
    wire.phase_invert = true;
    matrix.connect(wire);

    let mut engine = AudioMixerEngine::new(48000);
    *engine.matrix_mut() = matrix;
    let chunk = AudioChunk::new(vec![0.8, -0.4, 0.2], 1, 48000);
    engine.submit_input("src_tone", chunk);

    let output_chunk = engine.process_destination("dst_out");
    // Expected: 0.8 * 0.5 * -1.0 = -0.4; -0.4 * 0.5 * -1.0 = 0.2; 0.2 * 0.5 * -1.0 = -0.1
    assert!((output_chunk.samples[0] - (-0.4)).abs() < 1e-4);
    assert!((output_chunk.samples[1] - 0.2).abs() < 1e-4);
    assert!((output_chunk.samples[2] - (-0.1)).abs() < 1e-4);
}

/// @brief Tests dedicated monitor nodes and mute-when-monitoring behavior.
/// @return Returns `()` on assertion success.
#[test]
fn test_mute_when_monitoring_behavior() {
    let mut matrix = RoutingMatrix::new();
    let app_spotify = MixerNode::new("app_spotify", "Spotify", 0, 2);
    let monitor_headphones = MixerNode::new("out_headphones", "Headphones Monitor", 2, 0)
        .with_monitor(true)
        .with_mute_when_monitoring(true);
    let stream_virtual = MixerNode::new("out_stream", "OBS Stream Out", 2, 0);

    matrix.add_node(app_spotify);
    matrix.add_node(monitor_headphones);
    matrix.add_node(stream_virtual);

    matrix.connect(ConnectionWire::new(
        "wire_stream_0",
        AudioPinId::new("app_spotify", 0),
        AudioPinId::new("out_stream", 0),
        1.0,
    ));
    matrix.connect(
        ConnectionWire::new(
            "wire_monitor_0",
            AudioPinId::new("app_spotify", 0),
            AudioPinId::new("out_headphones", 0),
            1.0,
        )
        .with_monitor(true),
    );

    let mut engine = AudioMixerEngine::new(48000);
    *engine.matrix_mut() = matrix;
    engine.submit_input("app_spotify", AudioChunk::new(vec![0.5, 0.5], 2, 48000));

    let monitor_out = engine.process_destination("out_headphones");
    assert!(!monitor_out.is_silent());

    let stream_out = engine.process_destination("out_stream");
    assert!(!stream_out.is_silent());
}

/// @brief Tests automatic redirection of audio streams to fallback device when primary device is offline.
/// @return Returns `()` on assertion success.
#[test]
fn test_fallback_routing_when_primary_offline() {
    let mut matrix = RoutingMatrix::new();
    let src = MixerNode::new("src_app", "Game Audio", 0, 2);
    let dst_primary = MixerNode::new("dst_usb_headset", "USB Headset", 2, 0);
    let dst_fallback = MixerNode::new("dst_speakers", "Default Speakers", 2, 0);

    matrix.add_node(src);
    matrix.add_node(dst_primary);
    matrix.add_node(dst_fallback);

    matrix.set_fallback_node_id(Some("dst_speakers".to_string()));
    matrix.connect(ConnectionWire::new(
        "wire_game_to_usb",
        AudioPinId::new("src_app", 0),
        AudioPinId::new("dst_usb_headset", 0),
        1.0,
    ));

    // Mark primary device offline (e.g. USB headset unplugged)
    matrix.set_node_offline("dst_usb_headset", true);

    let mut engine = AudioMixerEngine::new(48000);
    *engine.matrix_mut() = matrix;
    engine.submit_input("src_app", AudioChunk::new(vec![0.7, 0.7], 2, 48000));

    // Primary device is offline -> should render silence
    let primary_out = engine.process_destination("dst_usb_headset");
    assert!(primary_out.is_silent());

    // Fallback device receives rerouted audio automatically
    let fallback_out = engine.process_destination("dst_speakers");
    assert!(!fallback_out.is_silent());
    assert!((fallback_out.samples[0] - 0.7).abs() < 1e-4);
}
