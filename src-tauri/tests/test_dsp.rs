//! @file test_dsp.rs
//! @brief Unit and integration tests for real-time DSP noise gate and sidechain auto-ducking.
//! @details Validates sub-threshold attenuation and above-threshold transparency for NoiseGate,
//! as well as smooth voice-triggered sidechain attenuation for AutoDucker.

use dyno_virtual_audio_mixer_lib::audio::dsp::{AutoDucker, NoiseGate};

/// @brief Tests that NoiseGate silences quiet audio below threshold and passes loud audio.
/// @return Returns `()` on assertion success.
#[test]
fn test_noise_gate_attenuation_and_pass_through() {
    let mut gate = NoiseGate::new(-40.0, 1.0, 50.0, 48000);

    // Quiet buffer (-60 dBFS)
    let mut quiet_samples = vec![0.001f32; 480];
    gate.process(&mut quiet_samples);
    for sample in quiet_samples {
        assert!(sample.abs() < 1e-4, "Quiet signal should be gated to near zero");
    }

    // Loud buffer (-6 dBFS)
    let mut loud_samples = vec![0.5f32; 480];
    gate.process(&mut loud_samples);
    // After attack time, gain opens to ~1.0
    assert!(loud_samples[479] > 0.45, "Loud signal should pass through gate cleanly");
}

/// @brief Tests that disabling NoiseGate bypasses all processing cleanly.
/// @return Returns `()` on assertion success.
#[test]
fn test_noise_gate_bypass() {
    let mut gate = NoiseGate::new(-20.0, 1.0, 50.0, 48000);
    gate.set_enabled(false);

    let mut samples = vec![0.01f32; 100];
    gate.process(&mut samples);
    assert_eq!(samples[0], 0.01f32);
    assert_eq!(samples[99], 0.01f32);
}

/// @brief Tests that AutoDucker attenuates target audio when voice trigger exceeds threshold.
/// @return Returns `()` on assertion success.
#[test]
fn test_auto_ducking_attenuates_target_when_trigger_active() {
    let mut ducker = AutoDucker::new(
        "mic_host",
        vec!["app_spotify".to_string()],
        -30.0, // threshold dB
        -18.0, // duck reduction dB (linear ~0.125)
        5.0,   // attack ms
        100.0, // release ms
        48000,
    );

    let trigger_active = vec![0.4f32; 480]; // Loud voice ~ -8 dBFS
    let mut music_target = vec![0.8f32; 960]; // Loud music
    let target_id = "app_spotify".to_string();

    ducker.process_sidechain(&trigger_active, &mut [(&target_id, &mut music_target)]);
    let ducked_sample = music_target[959];
    assert!(
        ducked_sample < 0.25,
        "Target music should be ducked significantly (got {})",
        ducked_sample
    );

    // Now silence on trigger -> target volume should release back toward 0.8
    let trigger_silent = vec![0.0f32; 4800]; // 100ms of silence
    for _ in 0..10 {
        ducker.process_sidechain(&trigger_silent, &mut [(&target_id, &mut music_target)]);
    }
}
