//! @file test_capture.rs
//! @brief Unit and integration tests for live audio capture and peak meter tracking.

use dyno_virtual_audio_mixer_lib::audio::capture::{
    linear_to_db, process_f32_samples, process_i16_samples,
    spawn_microphone_capture_thread, LiveMeterTracker, METER_SILENCE_DB,
};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;

/// @brief Tests decibel conversion and boundary clamping behavior.
#[test]
fn test_linear_to_db_conversion() {
    assert_eq!(linear_to_db(0.0), METER_SILENCE_DB);
    assert_eq!(linear_to_db(0.00005), METER_SILENCE_DB);
    assert!((linear_to_db(1.0) - 0.0).abs() < 0.01);
    assert!((linear_to_db(0.5) - (-6.02)).abs() < 0.05);
    assert!((linear_to_db(2.0) - 0.0).abs() < 0.01); // Clamped to 0 dBFS
}

/// @brief Tests LiveMeterTracker instantaneous attack and decay ballistics.
#[test]
fn test_live_meter_tracker_attack_and_decay() {
    let tracker = LiveMeterTracker::new();
    let (init_l, init_r) = tracker.peek();
    assert_eq!(init_l, METER_SILENCE_DB);
    assert_eq!(init_r, METER_SILENCE_DB);

    // Instant attack jump
    tracker.update(-6.0, -12.0);
    let (l, r) = tracker.peek();
    assert!((l - (-6.0)).abs() < 0.01);
    assert!((r - (-12.0)).abs() < 0.01);

    // Lower readings do not pull down peak immediately (attack preserves higher)
    tracker.update(-20.0, -30.0);
    let (l2, r2) = tracker.peek();
    assert!((l2 - (-6.0)).abs() < 0.01);
    assert!((r2 - (-12.0)).abs() < 0.01);

    // Decay step
    let (decayed_l, decayed_r) = tracker.get_and_decay(1.5);
    assert!((decayed_l - (-6.0)).abs() < 0.01);
    assert!((decayed_r - (-12.0)).abs() < 0.01);

    let (after_l, after_r) = tracker.peek();
    assert!((after_l - (-7.5)).abs() < 0.01);
    assert!((after_r - (-13.5)).abs() < 0.01);
}

/// @brief Tests processing f32 audio buffers (blowing or speaking into microphone).
#[test]
fn test_process_f32_samples_peak_detection() {
    let tracker = LiveMeterTracker::new();

    // Buffer simulating a blow into a mono mic (0.89 amplitude)
    let mono_buffer = vec![0.05, 0.1, 0.4, 0.89, -0.7, 0.2];
    process_f32_samples(&mono_buffer, 1, &tracker);

    let (l, r) = tracker.peek();
    let expected_db = 20.0 * 0.89f32.log10();
    assert!((l - expected_db).abs() < 0.05);
    assert!((r - expected_db).abs() < 0.05);
}

/// @brief Tests processing 16-bit PCM integer samples.
#[test]
fn test_process_i16_samples_peak_detection() {
    let tracker = LiveMeterTracker::new();

    // 16384 is approx 0.5 amplitude (-6.02 dB)
    let buffer = vec![16384i16, 0, -16384, 0];
    process_i16_samples(&buffer, 1, &tracker);

    let (l, r) = tracker.peek();
    assert!((l - (-6.02)).abs() < 0.1);
    assert!((r - (-6.02)).abs() < 0.1);
}

/// @brief Tests live microphone stream creation (without panicking on headless or live hardware).
#[test]
fn test_microphone_capture_thread_start() {
    let tracker = Arc::new(LiveMeterTracker::new());
    let is_running = Arc::new(AtomicBool::new(true));
    let handle = spawn_microphone_capture_thread(tracker.clone(), is_running.clone());

    // Allow thread to spin up and sample live hardware
    std::thread::sleep(Duration::from_millis(100));

    // Stop thread cleanly
    is_running.store(false, std::sync::atomic::Ordering::Relaxed);
    let _ = handle.join();
}
