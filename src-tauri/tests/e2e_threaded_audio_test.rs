//! @file e2e_threaded_audio_test.rs
//! @brief End-to-end multi-threaded audio streaming and concurrent telemetry stress test.
//! @details Simulates a real-time producer streaming audio frames into a lock-free SPSC ring buffer,
//! while a mixer thread processes frames through AudioMixerEngine and an observer thread queries telemetry.

use dyno_virtual_audio_mixer_lib::audio::matrix::{
    AudioPinId, ConnectionWire, MixerNode, RoutingMatrix,
};
use dyno_virtual_audio_mixer_lib::audio::mixer::AudioMixerEngine;
use dyno_virtual_audio_mixer_lib::audio::ring_buffer::{AudioConsumer, AudioProducer, AudioRingBuffer};
use dyno_virtual_audio_mixer_lib::audio::types::{AudioChunk, MIN_PEAK_DB};
use dyno_virtual_audio_mixer_lib::audio::virtual_device::VirtualDeviceManager;
use dyno_virtual_audio_mixer_lib::commands::AppState;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// @brief Spawns a producer thread generating periodic audio frames into the ring buffer.
/// @param is_running Atomic boolean controlling thread lifecycle.
/// @param mut producer Lock-free SPSC audio producer endpoint.
/// @return JoinHandle returning the total number of frames pushed.
fn spawn_audio_producer(
    is_running: Arc<AtomicBool>,
    mut producer: AudioProducer,
) -> JoinHandle<usize> {
    thread::spawn(move || {
        let mut frame_chunk = Vec::with_capacity(128);
        for _ in 0..64 {
            frame_chunk.push(0.3f32);
            frame_chunk.push(-0.3f32);
        }

        let mut frames_sent = 0usize;
        while is_running.load(Ordering::Relaxed) && frames_sent < 500 {
            if producer.slots() >= frame_chunk.len() {
                producer.push_samples(&frame_chunk);
                frames_sent += 1;
            } else {
                thread::sleep(Duration::from_millis(1));
            }
        }
        frames_sent
    })
}

/// @brief Spawns a mixer worker thread processing audio frames from the ring buffer.
/// @param is_running Atomic boolean controlling thread lifecycle.
/// @param mut consumer Lock-free SPSC audio consumer endpoint.
/// @param state Shared application state container.
/// @return JoinHandle returning the total number of frames rendered.
fn spawn_mixer_consumer(
    is_running: Arc<AtomicBool>,
    mut consumer: AudioConsumer,
    state: AppState,
) -> JoinHandle<usize> {
    thread::spawn(move || {
        let mut mixer = AudioMixerEngine::new(48000);
        let mut read_buf = vec![0.0f32; 256];
        let mut frames_processed = 0usize;

        while is_running.load(Ordering::Relaxed) && frames_processed < 250 {
            let read = consumer.pop_samples(&mut read_buf);
            if read > 0 {
                let chunk = AudioChunk::new(read_buf[..read].to_vec(), 2, 48000);
                let (peak_l, peak_r) = chunk.calculate_peak_db();
                assert!(peak_l > MIN_PEAK_DB);
                assert!(peak_r > MIN_PEAK_DB);

                let matrix_lock = state.read_matrix().expect("Failed to read matrix");
                *mixer.matrix_mut() = matrix_lock.clone();
                drop(matrix_lock);

                mixer.submit_input("src_synth", chunk);
                let rendered = mixer.process_destination("dst_stream");
                assert_eq!(rendered.channels, 2);
                frames_processed += 1;
            } else {
                thread::sleep(Duration::from_millis(1));
            }
        }
        frames_processed
    })
}

/// @brief Spawns a meter telemetry reader thread observing peak levels at high frequency.
/// @param is_running Atomic boolean controlling thread lifecycle.
/// @param state Shared application state container.
/// @return JoinHandle returning the total number of telemetry snapshots collected.
fn spawn_meter_observer(
    is_running: Arc<AtomicBool>,
    state: AppState,
) -> JoinHandle<usize> {
    thread::spawn(move || {
        let mut snapshot_count = 0usize;
        while is_running.load(Ordering::Relaxed) && snapshot_count < 50 {
            let snapshot = state.generate_meter_snapshot();
            assert_eq!(snapshot.meters.len(), 2);
            snapshot_count += 1;
            thread::sleep(Duration::from_millis(2));
        }
        snapshot_count
    })
}

/// @brief Runs concurrent multi-threaded audio streaming and telemetry test.
/// @return Returns `()` on assertion success.
#[test]
fn test_e2e_threaded_streaming_and_concurrent_metering() {
    let mut matrix = RoutingMatrix::new();
    matrix.add_node(MixerNode::new("src_synth", "Synthesizer", 0, 2).with_volume(0.9));
    matrix.add_node(MixerNode::new("dst_stream", "OBS Stream", 2, 0).with_volume(1.0));
    matrix.connect(ConnectionWire::new("w1", AudioPinId::left("src_synth"), AudioPinId::left("dst_stream"), 1.0));
    matrix.connect(ConnectionWire::new("w2", AudioPinId::right("src_synth"), AudioPinId::right("dst_stream"), 1.0));

    let app_state = AppState::with_matrix_and_devices(matrix, VirtualDeviceManager::new());
    let ring_buffer = AudioRingBuffer::new(8192);
    let (producer, consumer) = ring_buffer.split();
    let is_running = Arc::new(AtomicBool::new(true));

    let prod_handle = spawn_audio_producer(Arc::clone(&is_running), producer);
    let cons_handle = spawn_mixer_consumer(Arc::clone(&is_running), consumer, app_state.clone());
    let obs_handle = spawn_meter_observer(Arc::clone(&is_running), app_state);

    thread::sleep(Duration::from_millis(150));
    is_running.store(false, Ordering::Relaxed);

    let sent = prod_handle.join().expect("Producer panicked");
    let processed = cons_handle.join().expect("Consumer panicked");
    let snapshots = obs_handle.join().expect("Observer panicked");

    assert!(sent > 0, "Producer should send frames");
    assert!(processed > 0, "Consumer should process frames");
    assert!(snapshots > 0, "Observer should capture snapshots");
}
