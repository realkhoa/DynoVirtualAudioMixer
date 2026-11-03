//! @file ducker.rs
//! @brief Real-time sidechain auto-ducking DSP processor.
//! @details Detects peak voice activity on a trigger channel (e.g. Host Microphone) and
//! automatically attenuates one or more target channels (e.g. Music, Game, Media) by a
//! configurable decibel reduction with smooth attack and release envelopes.

use serde::{Deserialize, Serialize};

/// @brief Configuration and state for a sidechain auto-ducker routing relationship.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutoDucker {
    /// @brief Unique identifier of the trigger node whose audio activity activates ducking.
    pub trigger_node_id: String,
    /// @brief List of target node IDs whose volume is attenuated when trigger is active.
    pub target_node_ids: Vec<String>,
    /// @brief Trigger voice threshold in dBFS (e.g. -35.0 dBFS).
    pub threshold_db: f32,
    /// @brief Amount of attenuation to apply to targets in dB (e.g. -15.0 dB).
    pub duck_attenuation_db: f32,
    /// @brief Attack ramp time in milliseconds (e.g. 15.0 ms).
    pub attack_ms: f32,
    /// @brief Release ramp time in milliseconds (e.g. 400.0 ms).
    pub release_ms: f32,
    /// @brief Audio sample rate in Hertz.
    pub sample_rate: u32,
    /// @brief Whether sidechain auto-ducking is active.
    pub enabled: bool,
    /// @brief Current smoothed linear gain multiplier applied to targets [attenuation, 1.0].
    #[serde(skip)]
    current_gain: f32,
}

impl AutoDucker {
    /// @brief Constructs a new AutoDucker instance.
    /// @param trigger_node_id Identifier of the voice trigger node.
    /// @param target_node_ids Vector of destination nodes to duck.
    /// @param threshold_db Decibel threshold to engage ducking.
    /// @param duck_attenuation_db Decibel attenuation factor (e.g. -15.0 dB).
    /// @param attack_ms Attack time in milliseconds.
    /// @param release_ms Release time in milliseconds.
    /// @param sample_rate Audio sample rate in Hertz.
    /// @return Initialized AutoDucker instance.
    pub fn new(
        trigger_node_id: impl Into<String>,
        target_node_ids: Vec<String>,
        threshold_db: f32,
        duck_attenuation_db: f32,
        attack_ms: f32,
        release_ms: f32,
        sample_rate: u32,
    ) -> Self {
        Self {
            trigger_node_id: trigger_node_id.into(),
            target_node_ids,
            threshold_db,
            duck_attenuation_db,
            attack_ms: attack_ms.max(0.1),
            release_ms: release_ms.max(1.0),
            sample_rate: sample_rate.max(8000),
            enabled: true,
            current_gain: 1.0,
        }
    }

    /// @brief Calculates the linear attenuation multiplier corresponding to duck_attenuation_db.
    /// @return Linear gain factor (e.g. -18.0 dB corresponds to ~0.125).
    #[inline]
    pub fn min_gain(&self) -> f32 {
        10.0f32.powf(self.duck_attenuation_db.min(0.0) / 20.0)
    }

    /// @brief Inspects trigger samples and applies smooth ducking attenuation to target buffers.
    /// @param trigger_samples Interleaved samples from the voice trigger node.
    /// @param target_buffers Mutable slice of target node ID and sample slice pairs.
    pub fn process_sidechain(
        &mut self,
        trigger_samples: &[f32],
        target_buffers: &mut [(&String, &mut [f32])],
    ) {
        if !self.enabled || target_buffers.is_empty() {
            return;
        }

        // Measure peak activity on trigger channel
        let mut peak_trigger = 0.0f32;
        for &s in trigger_samples {
            let abs_s = s.abs();
            if abs_s > peak_trigger {
                peak_trigger = abs_s;
            }
        }

        let threshold_linear = 10.0f32.powf(self.threshold_db / 20.0);
        let min_gain = self.min_gain();
        let target_gain = if peak_trigger >= threshold_linear { min_gain } else { 1.0f32 };

        let sr = self.sample_rate as f32;
        let buffer_duration_s = if trigger_samples.is_empty() {
            0.010f32
        } else {
            (trigger_samples.len() as f32) / sr
        };
        let time_constant_s = if target_gain < self.current_gain {
            self.attack_ms * 0.001
        } else {
            self.release_ms * 0.001
        };
        let coeff = (-buffer_duration_s / time_constant_s.max(1e-4)).exp();

        let start_gain = self.current_gain;
        let end_gain = target_gain + coeff * (start_gain - target_gain);
        self.current_gain = end_gain;

        // Apply smooth gain ramp to matching target buffers
        for (node_id, samples) in target_buffers.iter_mut() {
            if self.target_node_ids.iter().any(|t| t == *node_id) {
                let n = samples.len().max(1) as f32;
                for (i, sample) in samples.iter_mut().enumerate() {
                    let g = start_gain + (i as f32 / n) * (end_gain - start_gain);
                    *sample *= g;
                }
            }
        }
    }
}
