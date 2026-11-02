# Core Audio Engine - Lock-Free Ring Buffer & Buffer Types

## 1. Overview & Purpose
This module provides low-latency, real-time lock-free audio primitives for the Dyno Virtual Audio Mixer. It bridges audio capture threads (e.g. WASAPI capture callbacks) and playback/mixing threads without mutex contention, memory reallocation, or priority inversion.

Key responsibilities:
- **`AudioChunk`**: Interleaved 32-bit floating-point PCM audio chunk with channel count and sample rate metadata.
- **`AudioRingBuffer` / `AudioProducer` / `AudioConsumer`**: Real-time wait-free Single-Producer Single-Consumer (SPSC) ring buffer for streaming individual audio samples between capture and mixing loops.
- **`ChunkRingBuffer` / `ChunkProducer` / `ChunkConsumer`**: SPSC ring buffer for queuing discrete `AudioChunk` structures.
- **Peak Metering**: Fast peak decibel level computation (`calculate_peak_db`) with -60.0 dB minimum floor and 0.0 dB full-scale normalization.
- **Soft Clipping / Limiter**: Hyperbolic tangent (`tanh`) saturation to smoothly protect against hard digital clipping during multi-track summing.

## 2. Prerequisites & Dependencies
- **Rust Edition**: 2021
- **Crate Dependencies**:
  - `rtrb = "0.3"`: Real-time safe, wait-free Single-Producer Single-Consumer ring buffer.
  - `serde = "1"`: Serialization/deserialization for audio chunk metadata.

## 3. Architecture & API Reference

### Primitives (`types.rs`)
| Type / Function | Description |
|---|---|
| `ChannelCount` | Enum representing `Mono` (1) or `Stereo` (2) audio stream formats. |
| `SampleRate` | Newtype wrapper around `u32` with standard frequencies (44.1 kHz, 48 kHz, 96 kHz, 192 kHz). |
| `AudioChunk` | Holds `samples: Vec<f32>`, `channels: u16`, `sample_rate: u32`. Provides `calculate_peak_db()`, `soft_clip()`, and `limit()`. |
| `calculate_peak_db(samples: &[f32], channels: u16) -> (f32, f32)` | Computes Left/Right peak dBFS values in `[MIN_PEAK_DB (-60.0), MAX_PEAK_DB (0.0)]`. |
| `MIN_PEAK_DB` | Constant `-60.0` dB representing digital silence or metering baseline. |
| `MAX_PEAK_DB` | Constant `0.0` dB representing 0 dBFS full scale. |

### Ring Buffer (`ring_buffer.rs`)
| Type | Methods | Description |
|---|---|---|
| `AudioRingBuffer` | `new(capacity)`, `split()`, `push_samples()`, `pop_samples()`, `push_chunk()`, `pop_chunk()` | Unified or splittable sample-level SPSC ring buffer. |
| `AudioProducer` | `push_samples(&[f32]) -> usize`, `push_chunk(&AudioChunk) -> usize`, `slots()`, `is_full()` | Producer endpoint safe to move to capture thread. |
| `AudioConsumer` | `pop_samples(&mut [f32]) -> usize`, `pop_chunk(frames, ch, rate) -> Option<AudioChunk>`, `slots()`, `is_empty()` | Consumer endpoint safe to move to mixer thread. |
| `ChunkRingBuffer` | `new(capacity)`, `split()`, `push(chunk)`, `pop() -> Option<AudioChunk>` | High-level ring buffer for discrete `AudioChunk`s. |

## 4. Usage Examples

### Streaming Samples Across Threads
```rust
use dyno_virtual_audio_mixer_lib::audio::ring_buffer::AudioRingBuffer;
use std::thread;

let ring_buffer = AudioRingBuffer::new(4096);
let (mut producer, mut consumer) = ring_buffer.split();

// Thread 1: WASAPI capture callback
thread::spawn(move || {
    let captured_samples = [0.1f32, -0.1, 0.2, -0.2];
    let written = producer.push_samples(&captured_samples);
});

// Thread 2: Mixer or playback thread
thread::spawn(move || {
    let mut playback_buffer = [0.0f32; 4];
    let read = consumer.pop_samples(&mut playback_buffer);
});
```

### Metering and Soft Clipping
```rust
use dyno_virtual_audio_mixer_lib::audio::types::{AudioChunk, MIN_PEAK_DB};

let mut chunk = AudioChunk::new(vec![1.5, -2.0, 0.5, -0.5], 2, 48000);

// Prevent digital clipping by applying smooth saturation
chunk.soft_clip();

// Calculate peak levels for GUI volume meters
let (left_db, right_db) = chunk.calculate_peak_db();
assert!(left_db <= 0.0 && left_db >= MIN_PEAK_DB);
```

## 5. Output Structure & Schemas
- `AudioChunk.samples`: Interleaved array formatted as `[L0, R0, L1, R1, ...]`.
- Peak dBFS range: `[-60.0, 0.0]`. Silence clamps directly to `-60.0 dB`.

## 6. Troubleshooting & Edge Cases
- **Buffer Full**: `push_samples` returns the count of samples actually written without blocking. If count < slice length, audio capture overflow occurred. The caller should increment an underrun/overflow counter and avoid sleeping or blocking the audio thread.
- **Buffer Empty**: `pop_samples` returns the count of samples read. If insufficient samples are available, fill remaining frames with silence (0.0) to prevent audio stutter.
