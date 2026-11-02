# Core Audio Engine - Lock-Free Ring Buffer, Device Enumeration & Process Audio Detection

## 1. Overview & Purpose
This module provides low-latency, real-time audio primitives, device discovery, and active process audio session detection for the Dyno Virtual Audio Mixer. It bridges audio capture threads (WASAPI capture callbacks) and playback/mixing threads without mutex contention, memory reallocation, or priority inversion, while enabling dynamic discovery of hardware endpoints and software audio sessions.

Key responsibilities:
- **`AudioChunk`**: Interleaved 32-bit floating-point PCM audio chunk with channel count and sample rate metadata.
- **`AudioRingBuffer` / `AudioProducer` / `AudioConsumer`**: Real-time wait-free Single-Producer Single-Consumer (SPSC) ring buffer for streaming individual audio samples between capture and mixing loops.
- **`ChunkRingBuffer` / `ChunkProducer` / `ChunkConsumer`**: SPSC ring buffer for queuing discrete `AudioChunk` structures.
- **Peak Metering**: Fast peak decibel level computation (`calculate_peak_db`) with -60.0 dB minimum floor and 0.0 dB full-scale normalization.
- **Soft Clipping / Limiter**: Hyperbolic tangent (`tanh`) saturation to smoothly protect against hard digital clipping during multi-track summing.
- **`list_audio_devices()`**: Enumerates all physical and virtual input and output audio endpoints via CPAL/WASAPI, extracting channels, sample rates, and default flags.
- **`list_audio_processes()`**: Scans active Windows WASAPI audio sessions, identifies running applications producing or capable of producing audio (Spotify, Discord, browsers, games), and associates them with PIDs and window titles.

## 2. Prerequisites & Dependencies
- **Rust Edition**: 2021
- **Crate Dependencies**:
  - `rtrb = "0.3"`: Real-time safe, wait-free Single-Producer Single-Consumer ring buffer.
  - `serde = "1"`: Serialization/deserialization for audio chunk and device metadata.
  - `cpal = "0.15"`: Cross-platform audio I/O library for physical and virtual device querying.
  - `windows = "0.58"`: Direct Windows Win32 API access for WASAPI audio session enumeration (`IMMDeviceEnumerator`, `IAudioSessionManager2`, `IAudioSessionControl2`), process querying, and window titles.

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

### Device Enumeration (`devices.rs`)
| Type / Function | Description |
|---|---|
| `AudioDeviceType` | Enum (`Input`, `Output`, `Virtual`) with helper predicate methods (`is_input()`, `is_output()`, `is_virtual()`). |
| `AudioDeviceDescriptor` | Struct containing `id: String`, `name: String`, `device_type: AudioDeviceType`, `channels: u16`, `sample_rate: u32`, `is_default: bool`. |
| `list_audio_devices() -> Result<Vec<AudioDeviceDescriptor>, String>` | Scans host system for audio capture and render endpoints using CPAL. |

### Process Audio Detection (`process_scanner.rs`)
| Type / Function | Description |
|---|---|
| `AudioProcessDescriptor` | Struct containing `pid: u32`, `name: String`, `title: String`, `has_audio_session: bool`. |
| `list_audio_processes() -> Result<Vec<AudioProcessDescriptor>, String>` | Scans WASAPI active audio sessions and application windows to discover audio-producing processes. |
| `is_audio_candidate_application(name: &str) -> bool` | Heuristic predicate identifying well-known audio/media applications (Spotify, Discord, Chrome, etc.). |

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

### Enumerating Audio Devices
```rust
use dyno_virtual_audio_mixer_lib::audio::devices::list_audio_devices;

let devices = list_audio_devices().expect("Failed to enumerate audio devices");
for dev in devices {
    println!(
        "[{:?}] {} (channels={}, rate={}Hz, default={})",
        dev.device_type, dev.name, dev.channels, dev.sample_rate, dev.is_default
    );
}
```

### Detecting Active Audio Processes
```rust
use dyno_virtual_audio_mixer_lib::audio::process_scanner::list_audio_processes;

let processes = list_audio_processes().expect("Failed to scan audio processes");
for proc in processes {
    if proc.has_audio_session {
        println!("Active Audio Session: PID {} - {} ({})", proc.pid, proc.name, proc.title);
    }
}
```

## 5. Output Structure & Schemas
- `AudioChunk.samples`: Interleaved array formatted as `[L0, R0, L1, R1, ...]`.
- Peak dBFS range: `[-60.0, 0.0]`. Silence clamps directly to `-60.0 dB`.
- `AudioDeviceDescriptor`:
  ```json
  {
    "id": "out-0-speakers-usb-condenser-microphone",
    "name": "Speakers (USB Condenser Microphone)",
    "device_type": "Output",
    "channels": 2,
    "sample_rate": 48000,
    "is_default": true
  }
  ```
- `AudioProcessDescriptor`:
  ```json
  {
    "pid": 9956,
    "name": "Discord.exe",
    "title": "Discord",
    "has_audio_session": true
  }
  ```

## 6. Troubleshooting & Edge Cases
- **Buffer Full**: `push_samples` returns the count of samples actually written without blocking. If count < slice length, audio capture overflow occurred. The caller should increment an underrun/overflow counter and avoid sleeping or blocking the audio thread.
- **Buffer Empty**: `pop_samples` returns the count of samples read. If insufficient samples are available, fill remaining frames with silence (0.0) to prevent audio stutter.
- **Headless Environments / Missing Hardware**: In containers or CI environments without physical audio hardware, `list_audio_devices()` returns an empty vector without panicking.
- **Permissions / Protected System Processes**: Protected Windows services (e.g. `audiodg.exe` or SYSTEM processes) may refuse `OpenProcess` calls. `get_process_name_by_pid` falls back to placeholder names like `"Process-<PID>"` or `"System"` rather than failing.
