//! @file capture.rs
//! @brief Live hardware microphone capture worker and lock-free real-time audio peak detector.
//! @details Spawns a dedicated thread owning the CPAL WASAPI input stream, continuously
//! samples live PCM audio buffers on the audio thread, and writes real-time peak decibels
//! into an atomic lock-free tracker for responsive VU metering and patchbay telemetry.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// @brief Floor decibel level representing digital silence.
pub const METER_SILENCE_DB: f32 = -60.0;

/// @brief Converts a linear amplitude scale (0.0 to 1.0+) to decibels full-scale (dBFS).
/// @param linear Linear peak amplitude value.
/// @return Clamped decibel level between -60.0 dBFS and 0.0 dBFS.
#[inline]
pub fn linear_to_db(linear: f32) -> f32 {
    if linear <= 0.0001 {
        METER_SILENCE_DB
    } else {
        (20.0 * linear.log10()).clamp(METER_SILENCE_DB, 0.0)
    }
}

/// @brief Lock-free real-time peak meter tracker updated from the CPAL audio callback thread.
/// @details Uses AtomicU32 with IEEE 754 float bit-casting to avoid lock contention on the
/// high-priority audio callback thread. Thread-safe (`Send + Sync`).
pub struct LiveMeterTracker {
    /// @brief Left channel peak decibels stored as f32 raw bits.
    peak_left_bits: AtomicU32,
    /// @brief Right channel peak decibels stored as f32 raw bits.
    peak_right_bits: AtomicU32,
}

impl Default for LiveMeterTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl LiveMeterTracker {
    /// @brief Constructs a new LiveMeterTracker initialized at digital silence (-60 dBFS).
    /// @return Initialized LiveMeterTracker.
    pub fn new() -> Self {
        Self {
            peak_left_bits: AtomicU32::new(METER_SILENCE_DB.to_bits()),
            peak_right_bits: AtomicU32::new(METER_SILENCE_DB.to_bits()),
        }
    }

    /// @brief Updates current peak readings with instantaneous attack.
    /// @param db_left Measured left channel decibels.
    /// @param db_right Measured right channel decibels.
    pub fn update(&self, db_left: f32, db_right: f32) {
        let cur_l = f32::from_bits(self.peak_left_bits.load(Ordering::Relaxed));
        if db_left > cur_l {
            self.peak_left_bits
                .store(db_left.to_bits(), Ordering::Relaxed);
        }

        let cur_r = f32::from_bits(self.peak_right_bits.load(Ordering::Relaxed));
        if db_right > cur_r {
            self.peak_right_bits
                .store(db_right.to_bits(), Ordering::Relaxed);
        }
    }

    /// @brief Extracts current peak decibel readings and applies smooth exponential ballistics decay.
    /// @param decay_step_db Decibels to decrease per 60 FPS tick (e.g. 1.2 dB).
    /// @return Tuple containing (peak_left_db, peak_right_db).
    pub fn get_and_decay(&self, decay_step_db: f32) -> (f32, f32) {
        let cur_l = f32::from_bits(self.peak_left_bits.load(Ordering::Relaxed));
        let cur_r = f32::from_bits(self.peak_right_bits.load(Ordering::Relaxed));

        let next_l = (cur_l - decay_step_db).max(METER_SILENCE_DB);
        let next_r = (cur_r - decay_step_db).max(METER_SILENCE_DB);

        self.peak_left_bits
            .store(next_l.to_bits(), Ordering::Relaxed);
        self.peak_right_bits
            .store(next_r.to_bits(), Ordering::Relaxed);

        (cur_l, cur_r)
    }

    /// @brief Returns current peak readings without applying decay.
    /// @return Tuple containing (peak_left_db, peak_right_db).
    pub fn peek(&self) -> (f32, f32) {
        let cur_l = f32::from_bits(self.peak_left_bits.load(Ordering::Relaxed));
        let cur_r = f32::from_bits(self.peak_right_bits.load(Ordering::Relaxed));
        (cur_l, cur_r)
    }
}

/// @brief Processes an audio slice of f32 samples, calculating Left and Right channel peak decibels.
/// @param data Audio samples buffer.
/// @param channels Interleaved channel count (1 for mono, 2 for stereo).
/// @param tracker LiveMeterTracker to update.
pub fn process_f32_samples(data: &[f32], channels: usize, tracker: &LiveMeterTracker) {
    if data.is_empty() {
        return;
    }

    let mut peak_l = 0.0f32;
    let mut peak_r = 0.0f32;

    if channels <= 1 {
        for &sample in data {
            let abs = sample.abs();
            if abs > peak_l {
                peak_l = abs;
            }
        }
        peak_r = peak_l;
    } else {
        for frame in data.chunks_exact(channels) {
            let abs_l = frame[0].abs();
            let abs_r = frame[1].abs();
            if abs_l > peak_l {
                peak_l = abs_l;
            }
            if abs_r > peak_r {
                peak_r = abs_r;
            }
        }
    }

    let db_l = linear_to_db(peak_l);
    let db_r = linear_to_db(peak_r);
    tracker.update(db_l, db_r);
}

/// @brief Processes an audio slice of i16 samples, converting to normalized float and calculating peak decibels.
/// @param data 16-bit integer PCM samples buffer.
/// @param channels Interleaved channel count.
/// @param tracker LiveMeterTracker to update.
pub fn process_i16_samples(data: &[i16], channels: usize, tracker: &LiveMeterTracker) {
    if data.is_empty() {
        return;
    }

    let mut peak_l = 0.0f32;
    let mut peak_r = 0.0f32;

    if channels <= 1 {
        for &sample in data {
            let abs = (sample as f32 / 32768.0).abs();
            if abs > peak_l {
                peak_l = abs;
            }
        }
        peak_r = peak_l;
    } else {
        for frame in data.chunks_exact(channels) {
            let abs_l = (frame[0] as f32 / 32768.0).abs();
            let abs_r = (frame[1] as f32 / 32768.0).abs();
            if abs_l > peak_l {
                peak_l = abs_l;
            }
            if abs_r > peak_r {
                peak_r = abs_r;
            }
        }
    }

    let db_l = linear_to_db(peak_l);
    let db_r = linear_to_db(peak_r);
    tracker.update(db_l, db_r);
}

/// @brief Processes an audio slice of u16 samples, converting to normalized float and calculating peak decibels.
/// @param data 16-bit unsigned PCM samples buffer.
/// @param channels Interleaved channel count.
/// @param tracker LiveMeterTracker to update.
pub fn process_u16_samples(data: &[u16], channels: usize, tracker: &LiveMeterTracker) {
    if data.is_empty() {
        return;
    }

    let mut peak_l = 0.0f32;
    let mut peak_r = 0.0f32;

    if channels <= 1 {
        for &sample in data {
            let abs = ((sample as f32 - 32768.0) / 32768.0).abs();
            if abs > peak_l {
                peak_l = abs;
            }
        }
        peak_r = peak_l;
    } else {
        for frame in data.chunks_exact(channels) {
            let abs_l = ((frame[0] as f32 - 32768.0) / 32768.0).abs();
            let abs_r = ((frame[1] as f32 - 32768.0) / 32768.0).abs();
            if abs_l > peak_l {
                peak_l = abs_l;
            }
            if abs_r > peak_r {
                peak_r = abs_r;
            }
        }
    }

    let db_l = linear_to_db(peak_l);
    let db_r = linear_to_db(peak_r);
    tracker.update(db_l, db_r);
}

/// @brief Spawns a dedicated thread to initialize and own the CPAL WASAPI input stream.
/// @details Isolates the non-Send CPAL Stream instance inside its dedicated worker thread,
/// feeding live decibel measurements into the shared `LiveMeterTracker`.
/// @param tracker Shared LiveMeterTracker instance.
/// @param is_running Atomic boolean controlling the capture thread lifecycle.
/// @return `JoinHandle<()>` of the spawned capture worker thread.
pub fn spawn_microphone_capture_thread(
    tracker: Arc<LiveMeterTracker>,
    is_running: Arc<AtomicBool>,
) -> thread::JoinHandle<()> {
    thread::Builder::new()
        .name("dyno-mic-capture".to_string())
        .spawn(move || {
            let host = cpal::default_host();
            let device = match host.default_input_device() {
                Some(d) => d,
                None => {
                    eprintln!("[AudioCapture] No default input audio device found on host.");
                    return;
                }
            };

            let dev_name = device
                .name()
                .unwrap_or_else(|_| "Default Microphone".to_string());
            println!("[AudioCapture] Starting live WASAPI capture on: {dev_name}");

            let config = match device.default_input_config() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("[AudioCapture] Failed to get default input config: {e}");
                    return;
                }
            };

            let channels = config.channels() as usize;
            let sample_format = config.sample_format();
            let stream_config: cpal::StreamConfig = config.into();

            let tracker_clone = tracker.clone();
            let err_fn = move |err| {
                eprintln!("[AudioCapture] Stream error: {err}");
            };

            let stream_res = match sample_format {
                cpal::SampleFormat::F32 => device.build_input_stream(
                    &stream_config,
                    move |data: &[f32], _: &_| {
                        process_f32_samples(data, channels, &tracker_clone);
                    },
                    err_fn,
                    None,
                ),
                cpal::SampleFormat::I16 => device.build_input_stream(
                    &stream_config,
                    move |data: &[i16], _: &_| {
                        process_i16_samples(data, channels, &tracker_clone);
                    },
                    err_fn,
                    None,
                ),
                cpal::SampleFormat::U16 => device.build_input_stream(
                    &stream_config,
                    move |data: &[u16], _: &_| {
                        process_u16_samples(data, channels, &tracker_clone);
                    },
                    err_fn,
                    None,
                ),
                other => {
                    eprintln!("[AudioCapture] Unsupported format: {other:?}");
                    return;
                }
            };

            let stream = match stream_res {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[AudioCapture] Failed to build CPAL input stream: {e}");
                    return;
                }
            };

            if let Err(e) = stream.play() {
                eprintln!("[AudioCapture] Failed to start stream: {e}");
                return;
            }

            println!("[AudioCapture] Live microphone capture active and streaming.");

            while is_running.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(50));
            }

            drop(stream);
            println!("[AudioCapture] Stream stopped cleanly.");
        })
        .expect("Failed to spawn dyno-mic-capture thread")
}
