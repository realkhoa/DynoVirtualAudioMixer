//! @file types.rs
//! @brief Fundamental audio engine primitives, channel formats, and metering types.
//! @details Provides `AudioChunk`, `ChannelCount`, `SampleRate`, and peak decibel
//! metering algorithms with zero-allocation soft-clipping protection for the Dyno audio mixer.

use serde::{Deserialize, Serialize};

/// @brief Minimum decibel floor representing digital silence or sub-audible noise (-60.0 dB).
pub const MIN_PEAK_DB: f32 = -60.0;

/// @brief Maximum decibel ceiling representing 0 dBFS full scale digital audio (0.0 dB).
pub const MAX_PEAK_DB: f32 = 0.0;

/// @brief Audio channel configuration for playback and capture streams.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChannelCount {
    /// @brief Single mono channel.
    Mono = 1,
    /// @brief Dual stereo channels (Left, Right).
    Stereo = 2,
}

impl ChannelCount {
    /// @brief Returns the numeric channel count.
    /// @return Number of audio channels as `u16` (1 for Mono, 2 for Stereo).
    #[inline]
    pub fn channels(&self) -> u16 {
        *self as u16
    }

    /// @brief Checks whether the configuration is Stereo.
    /// @return `true` if channel count is 2 (Stereo), `false` otherwise.
    #[inline]
    pub fn is_stereo(&self) -> bool {
        matches!(self, ChannelCount::Stereo)
    }

    /// @brief Checks whether the configuration is Mono.
    /// @return `true` if channel count is 1 (Mono), `false` otherwise.
    #[inline]
    pub fn is_mono(&self) -> bool {
        matches!(self, ChannelCount::Mono)
    }
}

impl TryFrom<u16> for ChannelCount {
    type Error = &'static str;

    /// @brief Attempts to construct a `ChannelCount` from an integer channel count.
    /// @param value Integer representation of channel count.
    /// @return Ok(ChannelCount) if valid (1 or 2), Err with description otherwise.
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(ChannelCount::Mono),
            2 => Ok(ChannelCount::Stereo),
            _ => Err("Unsupported channel count; only Mono (1) and Stereo (2) are supported"),
        }
    }
}

impl From<ChannelCount> for u16 {
    /// @brief Converts a `ChannelCount` to its raw `u16` value.
    /// @param count The `ChannelCount` instance.
    /// @return The channel count as `u16`.
    #[inline]
    fn from(count: ChannelCount) -> Self {
        count.channels()
    }
}

impl std::fmt::Display for ChannelCount {
    /// @brief Formats the channel count for human-readable display.
    /// @param f Formatter target.
    /// @return Formatting result.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChannelCount::Mono => write!(f, "Mono (1 ch)"),
            ChannelCount::Stereo => write!(f, "Stereo (2 ch)"),
        }
    }
}

/// @brief Standard audio sample rate representation in Hertz.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SampleRate(pub u32);

impl SampleRate {
    /// @brief Standard CD audio sample rate (44,100 Hz).
    pub const RATE_44100: SampleRate = SampleRate(44100);

    /// @brief Broadcast and video standard audio sample rate (48,000 Hz).
    pub const RATE_48000: SampleRate = SampleRate(48000);

    /// @brief High-resolution studio sample rate (96,000 Hz).
    pub const RATE_96000: SampleRate = SampleRate(96000);

    /// @brief Ultra-high resolution audio sample rate (192,000 Hz).
    pub const RATE_192000: SampleRate = SampleRate(192000);

    /// @brief Constructs a new `SampleRate` from a raw Hertz value.
    /// @param hz Frequency in Hertz.
    /// @return A new `SampleRate` instance.
    #[inline]
    pub const fn new(hz: u32) -> Self {
        SampleRate(hz)
    }

    /// @brief Returns the raw sample rate frequency in Hertz.
    /// @return Frequency as `u32`.
    #[inline]
    pub const fn hz(&self) -> u32 {
        self.0
    }

    /// @brief Validates if the sample rate is within normal professional audio bounds (8000 Hz to 384000 Hz).
    /// @return `true` if valid, `false` otherwise.
    #[inline]
    pub fn is_valid(&self) -> bool {
        (8000..=384000).contains(&self.0)
    }
}

impl From<u32> for SampleRate {
    /// @brief Converts a `u32` frequency value into a `SampleRate`.
    /// @param hz Frequency in Hertz.
    /// @return Corresponding `SampleRate`.
    #[inline]
    fn from(hz: u32) -> Self {
        SampleRate(hz)
    }
}

impl From<SampleRate> for u32 {
    /// @brief Converts a `SampleRate` into its numeric Hertz value.
    /// @param rate `SampleRate` instance.
    /// @return Frequency in Hertz as `u32`.
    #[inline]
    fn from(rate: SampleRate) -> Self {
        rate.0
    }
}

impl std::fmt::Display for SampleRate {
    /// @brief Formats the sample rate for human-readable display.
    /// @param f Formatter target.
    /// @return Formatting result.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} Hz", self.0)
    }
}

/// @brief Computes peak audio levels in decibels for left and right channels.
/// @details Scans interleaved floating-point PCM audio samples and computes peak amplitude.
/// Applies `20 * log10(max(amplitude, 1e-3))` clamped to `[MIN_PEAK_DB, 0.0]`. Silence
/// or zero-length input yields `MIN_PEAK_DB` (-60.0 dB).
/// @param samples Interleaved PCM audio sample slice in range [-1.0, 1.0].
/// @param channels Number of audio channels (1 for Mono, 2 for Stereo).
/// @return A tuple of `(left_db, right_db)` representing peak level in decibels clamped to at least -60.0 dB.
pub fn calculate_peak_db(samples: &[f32], channels: u16) -> (f32, f32) {
    if samples.is_empty() || channels == 0 {
        return (MIN_PEAK_DB, MIN_PEAK_DB);
    }

    let ch = channels as usize;
    if ch == 1 {
        let mut max_val = 0.0f32;
        for &s in samples {
            let abs_s = s.abs();
            if abs_s > max_val {
                max_val = abs_s;
            }
        }

        let db = if max_val <= 0.001 {
            MIN_PEAK_DB
        } else {
            (20.0 * max_val.log10()).clamp(MIN_PEAK_DB, MAX_PEAK_DB)
        };
        (db, db)
    } else {
        let mut max_left = 0.0f32;
        let mut max_right = 0.0f32;

        for frame in samples.chunks_exact(ch) {
            let left_abs = frame[0].abs();
            if left_abs > max_left {
                max_left = left_abs;
            }
            let right_abs = frame[1].abs();
            if right_abs > max_right {
                max_right = right_abs;
            }
        }

        let left_db = if max_left <= 0.001 {
            MIN_PEAK_DB
        } else {
            (20.0 * max_left.log10()).clamp(MIN_PEAK_DB, MAX_PEAK_DB)
        };

        let right_db = if max_right <= 0.001 {
            MIN_PEAK_DB
        } else {
            (20.0 * max_right.log10()).clamp(MIN_PEAK_DB, MAX_PEAK_DB)
        };

        (left_db, right_db)
    }
}

/// @brief Represents a contiguous chunk of interleaved floating-point PCM audio samples.
/// @details Contains audio sample buffers along with channel count and sampling rate metadata.
/// Provides real-time audio manipulation methods such as peak decibel calculation and soft clipping.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioChunk {
    /// @brief Interleaved 32-bit floating-point audio samples in range [-1.0, 1.0].
    pub samples: Vec<f32>,
    /// @brief Number of channels in the chunk (1 for Mono, 2 for Stereo).
    pub channels: u16,
    /// @brief Sample rate in Hertz (e.g. 44100, 48000).
    pub sample_rate: u32,
}

impl AudioChunk {
    /// @brief Creates a new `AudioChunk` with the given sample buffer and format parameters.
    /// @param samples Interleaved 32-bit floating-point audio samples.
    /// @param channels Number of audio channels.
    /// @param sample_rate Sample rate in Hertz.
    /// @return A new `AudioChunk` instance.
    pub fn new(samples: Vec<f32>, channels: u16, sample_rate: u32) -> Self {
        Self {
            samples,
            channels,
            sample_rate,
        }
    }

    /// @brief Creates an `AudioChunk` filled with digital silence (zeros).
    /// @param frame_count Number of audio frames (each frame consists of `channels` samples).
    /// @param channels Number of audio channels.
    /// @param sample_rate Sample rate in Hertz.
    /// @return A new `AudioChunk` containing `frame_count * channels` zeroed samples.
    pub fn silence(frame_count: usize, channels: u16, sample_rate: u32) -> Self {
        let total_samples = frame_count.saturating_mul(channels as usize);
        Self {
            samples: vec![0.0f32; total_samples],
            channels,
            sample_rate,
        }
    }

    /// @brief Checks whether the chunk contains only digital silence (zeros).
    /// @return `true` if empty or all samples are zero, `false` otherwise.
    #[inline]
    pub fn is_silent(&self) -> bool {
        self.samples.is_empty() || self.samples.iter().all(|&s| s.abs() < 1e-6)
    }

    /// @brief Returns the number of complete audio frames in this chunk.
    /// @return Frame count calculated as `samples.len() / channels`.
    #[inline]
    pub fn frame_count(&self) -> usize {
        if self.channels == 0 {
            0
        } else {
            self.samples.len() / (self.channels as usize)
        }
    }

    /// @brief Returns whether the audio chunk contains zero samples.
    /// @return `true` if samples vector is empty, `false` otherwise.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// @brief Returns the duration of the audio chunk in seconds.
    /// @return Duration in fractional seconds.
    #[inline]
    pub fn duration_seconds(&self) -> f32 {
        if self.sample_rate == 0 {
            0.0
        } else {
            self.frame_count() as f32 / self.sample_rate as f32
        }
    }

    /// @brief Computes peak dBFS levels for left and right channels.
    /// @return Tuple of `(left_db, right_db)` where silence yields -60.0 dB and 1.0 yields 0.0 dB.
    #[inline]
    pub fn calculate_peak_db(&self) -> (f32, f32) {
        calculate_peak_db(&self.samples, self.channels)
    }

    /// @brief Applies smooth hyperbolic tangent (`tanh`) soft saturation to prevent harsh digital clipping.
    /// @details For high input signals (e.g. summing multiple mixed audio streams), `tanh(x)`
    /// transparently saturates the signal smoothly towards ±1.0 without hard waveform slicing.
    pub fn soft_clip(&mut self) {
        for sample in &mut self.samples {
            *sample = sample.tanh();
        }
    }

    /// @brief Limits the maximum absolute amplitude of all samples to a given threshold.
    /// @param threshold Absolute peak ceiling (typically 1.0).
    pub fn limit(&mut self, threshold: f32) {
        let ceiling = threshold.abs();
        for sample in &mut self.samples {
            *sample = sample.clamp(-ceiling, ceiling);
        }
    }
}
