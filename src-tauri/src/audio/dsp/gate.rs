//! @file gate.rs
//! @brief Real-time digital audio noise gate filter with smooth envelope follower.
//! @details Attenuates or silences audio signals below a user-configured decibel threshold
//! to eliminate background noise (room noise, fan noise, mechanical keyboard clicks) with
//! smooth exponential attack and release gain ramps preventing audio clicking.

use serde::{Deserialize, Serialize};

/// @brief Real-time digital noise gate processor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoiseGate {
    /// @brief Threshold in dBFS below which the gate attenuates audio.
    pub threshold_db: f32,
    /// @brief Attack time in milliseconds to open the gate when signal exceeds threshold.
    pub attack_ms: f32,
    /// @brief Release time in milliseconds to close the gate when signal drops below threshold.
    pub release_ms: f32,
    /// @brief Audio sample rate in Hertz.
    pub sample_rate: u32,
    /// @brief Whether the noise gate is actively processing audio.
    pub enabled: bool,
    /// @brief Current smoothed linear gain state [0.0, 1.0].
    #[serde(skip)]
    current_gain: f32,
}

impl NoiseGate {
    /// @brief Constructs a new NoiseGate instance.
    /// @param threshold_db Decibel threshold (e.g. -45.0 dBFS).
    /// @param attack_ms Attack time in milliseconds (e.g. 2.0 ms).
    /// @param release_ms Release time in milliseconds (e.g. 100.0 ms).
    /// @param sample_rate Audio sample rate in Hertz (e.g. 48000).
    /// @return Newly initialized NoiseGate instance.
    pub fn new(threshold_db: f32, attack_ms: f32, release_ms: f32, sample_rate: u32) -> Self {
        Self {
            threshold_db,
            attack_ms: attack_ms.max(0.1),
            release_ms: release_ms.max(1.0),
            sample_rate: sample_rate.max(8000),
            enabled: true,
            current_gain: 0.0,
        }
    }

    /// @brief Enables or disables the noise gate filter.
    /// @param enabled Whether the gate should be active.
    #[inline]
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.current_gain = 1.0;
        }
    }

    /// @brief Configures the threshold in decibels.
    /// @param threshold_db New threshold in dBFS (e.g. -50.0 to -20.0).
    #[inline]
    pub fn set_threshold_db(&mut self, threshold_db: f32) {
        self.threshold_db = threshold_db;
    }

    /// @brief Processes an interleaved slice of audio samples in-place.
    /// @details For each sample, calculates instantaneous amplitude against linear threshold.
    /// Smoothly adapts the internal envelope gain and multiplies samples to eliminate noise.
    /// @param samples Mutable slice of interleaved 32-bit floating-point audio samples.
    pub fn process(&mut self, samples: &mut [f32]) {
        if !self.enabled || samples.is_empty() {
            return;
        }

        let threshold_linear = 10.0f32.powf(self.threshold_db / 20.0);
        let sr = self.sample_rate as f32;
        let attack_coeff = (-1.0 / (self.attack_ms * 0.001 * sr).max(1.0)).exp();
        let release_coeff = (-1.0 / (self.release_ms * 0.001 * sr).max(1.0)).exp();

        for sample in samples.iter_mut() {
            let level = sample.abs();
            let target_gain = if level >= threshold_linear { 1.0f32 } else { 0.0f32 };

            if target_gain > self.current_gain {
                // Opening the gate (attack)
                self.current_gain = target_gain + attack_coeff * (self.current_gain - target_gain);
            } else {
                // Closing the gate (release)
                self.current_gain = target_gain + release_coeff * (self.current_gain - target_gain);
            }

            *sample *= self.current_gain;
        }
    }
}
