# Core Audio Engine - Lock-Free Ring Buffer, Device Enumeration, Routing Matrix, Virtual Device Manager & Real-Time Mixer Pipeline

## 1. Overview & Purpose
This module provides low-latency, real-time audio primitives, device discovery, active process audio session detection, virtual audio device management (VB-Cable, Virtual Audio Cable, Software Loopback), pin-level routing matrix graph evaluation, and multi-source soft-clipping audio mixing for the Dyno Virtual Audio Mixer. It bridges audio capture threads (WASAPI capture callbacks) and playback/mixing threads without mutex contention, memory reallocation, or priority inversion, while enabling dynamic discovery of hardware endpoints, software audio sessions, virtual audio cables, and arbitrary pin-to-pin audio patching.

Key responsibilities:
- **`AudioChunk`**: Interleaved 32-bit floating-point PCM audio chunk with channel count and sample rate metadata.
- **`AudioRingBuffer` / `AudioProducer` / `AudioConsumer`**: Real-time wait-free Single-Producer Single-Consumer (SPSC) ring buffer for streaming individual audio samples between capture and mixing loops.
- **`ChunkRingBuffer` / `ChunkProducer` / `ChunkConsumer`**: SPSC ring buffer for queuing discrete `AudioChunk` structures.
- **Peak Metering**: Fast peak decibel level computation (`calculate_peak_db`) with -60.0 dB minimum floor and 0.0 dB full-scale normalization.
- **Soft Clipping / Limiter**: Hyperbolic tangent (`tanh`) saturation to smoothly protect against hard digital clipping during multi-track summing.
- **`list_audio_devices()`**: Enumerates all physical and virtual input and output audio endpoints via CPAL/WASAPI, extracting channels, sample rates, and default flags.
- **`list_audio_processes()`**: Scans active Windows WASAPI audio sessions, identifies running applications producing or capable of producing audio (Spotify, Discord, browsers, games), and associates them with PIDs and window titles.
- **`VirtualCableType` / `VirtualDeviceDescriptor` / `VirtualDeviceManager`**: Manages virtual audio devices on Windows, supporting VB-Audio Cable, Virtual Audio Cable (VAC), and software loopback endpoints for zero-hardware audio routing.
- **`AudioPinId` / `ConnectionWire` / `MixerNode`**: Topological primitives for graph-based audio patchbays with node volumes, wire gains, and granular muting.
- **`RoutingMatrix`**: Arbitrary multichannel routing matrix handling 1-to-1 direct routing, stereo downmixing, mono channel splitting, volume scaling, muting, and smooth soft saturation.
- **`AudioMixerEngine`**: Central mixing pipeline coordinating staged source buffers and matrix evaluation to produce destination render frames.
- **`LiveMeterTracker` / `spawn_microphone_capture_thread`**: Thread-isolated live WASAPI microphone capture service continuously reading physical microphone input streams via CPAL and updating an atomic lock-free peak meter tracker for real-time VU metering.

## 2. Prerequisites & Dependencies
- **Rust Edition**: 2021
- **Crate Dependencies**:
  - `rtrb = "0.3"`: Real-time safe, wait-free Single-Producer Single-Consumer ring buffer.
  - `serde = "1"`: Serialization/deserialization for audio chunk, wire, node, and device metadata.
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

### Device Enumeration & Process Audio Detection (`devices.rs`, `process_scanner.rs`)
| Type / Function | Description |
|---|---|
| `AudioDeviceType` | Enum (`Input`, `Output`, `Virtual`) with helper predicate methods (`is_input()`, `is_output()`, `is_virtual()`). |
| `AudioDeviceDescriptor` | Struct containing `id: String`, `name: String`, `device_type: AudioDeviceType`, `channels: u16`, `sample_rate: u32`, `is_default: bool`. |
| `list_audio_devices() -> Result<Vec<AudioDeviceDescriptor>, String>` | Scans host system for audio capture and render endpoints using CPAL. |
| `AudioProcessDescriptor` | Struct containing `pid: u32`, `name: String`, `title: String`, `has_audio_session: bool`. |
| `list_audio_processes() -> Result<Vec<AudioProcessDescriptor>, String>` | Scans WASAPI active audio sessions and application windows to discover audio-producing processes. |
| `is_audio_candidate_application(name: &str) -> bool` | Heuristic predicate identifying well-known audio/media applications (Spotify, Discord, Chrome, etc.). |

### Virtual Audio Device Adapter (`virtual_device.rs`)
| Type / Function | Description |
|---|---|
| `VirtualCableType` | Enum (`VBCable`, `VirtualAudioCable`, `SoftwareLoopbackEndpoint`) with display names and driver classification. |
| `VirtualDeviceDescriptor` | Metadata struct representing an instantiated virtual audio device/cable with endpoint IDs, channels, and sample rate. |
| `VirtualDeviceManager` | Manager for detecting installed third-party drivers (`detect_available_drivers`), creating named cables (`create_virtual_device`), auto-disambiguating names, toggling active states, and removing devices. |

### Routing Matrix & Real-Time Mixer Pipeline (`matrix.rs`, `mixer.rs`)
| Type / Function | Description |
|---|---|
| `AudioPinId` | Identifies a specific channel pin (`node_id: String`, `channel_index: u16`). |
| `ConnectionWire` | Represents virtual audio wire (`id: String`, `source: AudioPinId`, `destination: AudioPinId`, `gain: f32`, `muted: bool`). |
| `MixerNode` | Represents audio source, sub-mix, or output node (`id: String`, `name: String`, `volume: f32`, `muted: bool`, `input_channels: u16`, `output_channels: u16`). |
| `RoutingMatrix` | Graph manager with `add_node`, `remove_node`, `connect`, `disconnect`, `process_frame`. Handles gain scaling, muting, channel mapping, and soft saturation. |
| `AudioMixerEngine` | Pipeline coordinating input staging buffers and routing matrix evaluation (`submit_input`, `process_destination`, `render_all`, `clear_inputs`). |

### Live Microphone Capture & Real-Time Metering (`capture.rs`)
| Type / Function | Description |
|---|---|
| `LiveMeterTracker` | Lock-free peak meter tracker using `AtomicU32` float bitcasting (`update`, `get_and_decay`, `peek`). |
| `spawn_microphone_capture_thread(tracker, is_running)` | Spawns dedicated thread owning CPAL WASAPI input stream, streaming live mic PCM buffers into `LiveMeterTracker`. |
| `process_f32_samples(data, channels, tracker)` | Analyzes f32 PCM samples, calculates peak amplitude, converts to dBFS, updates tracker with instant attack. |
| `process_i16_samples(data, channels, tracker)` | Analyzes normalized 16-bit PCM integer samples and updates tracker. |
| `process_u16_samples(data, channels, tracker)` | Analyzes normalized 16-bit unsigned PCM samples and updates tracker. |
| `linear_to_db(linear) -> f32` | Converts linear amplitude scale (0.0 to 1.0) into clamped decibels `[-60.0, 0.0]`. |

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

### Pin-Level Routing Matrix Evaluation
```rust
use dyno_virtual_audio_mixer_lib::audio::matrix::{AudioPinId, ConnectionWire, MixerNode, RoutingMatrix};
use dyno_virtual_audio_mixer_lib::audio::types::AudioChunk;
use std::collections::HashMap;

let mut matrix = RoutingMatrix::new();
matrix.add_node(MixerNode::new("spotify", "Spotify", 0, 2).with_volume(1.0));
matrix.add_node(MixerNode::new("speakers", "Master Speakers", 2, 2).with_volume(0.8));

// Connect stereo channels L->L and R->R
matrix.connect(ConnectionWire::new("wire_l", AudioPinId::left("spotify"), AudioPinId::left("speakers"), 1.0));
matrix.connect(ConnectionWire::new("wire_r", AudioPinId::right("spotify"), AudioPinId::right("speakers"), 1.0));

let mut inputs = HashMap::new();
inputs.insert("spotify".to_string(), AudioChunk::new(vec![0.5, 0.5], 2, 48000));

let rendered_chunk = matrix.process_frame(&inputs, "speakers");
```

### AudioMixerEngine Pipeline
```rust
use dyno_virtual_audio_mixer_lib::audio::mixer::AudioMixerEngine;
use dyno_virtual_audio_mixer_lib::audio::matrix::{AudioPinId, ConnectionWire, MixerNode};
use dyno_virtual_audio_mixer_lib::audio::types::AudioChunk;

let mut engine = AudioMixerEngine::new(48000);
engine.add_node(MixerNode::new("mic", "Microphone", 0, 1));
engine.add_node(MixerNode::new("stream_out", "Stream Bus", 2, 2));

// Split mono microphone to both stereo channels
engine.connect(ConnectionWire::new("m_l", AudioPinId::new("mic", 0), AudioPinId::left("stream_out"), 1.0));
engine.connect(ConnectionWire::new("m_r", AudioPinId::new("mic", 0), AudioPinId::right("stream_out"), 1.0));

engine.submit_input("mic", AudioChunk::new(vec![0.75], 1, 48000));
let stream_chunk = engine.process_destination("stream_out");
```

### Managing Virtual Audio Cables
```rust
use dyno_virtual_audio_mixer_lib::audio::virtual_device::{
    VirtualCableType, VirtualDeviceManager,
};

let mut manager = VirtualDeviceManager::new();

// Discover available hardware drivers or fallback to software loopback
let drivers = manager.detect_available_drivers();

// Create a named virtual cable for Discord
let discord_cable = manager
    .create_virtual_device("Discord Aux", VirtualCableType::SoftwareLoopbackEndpoint)
    .expect("Failed to create virtual cable");

// Check active status or list all devices
let all_devices = manager.list_devices();
manager.set_device_active(&discord_cable.id, false).ok();
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
- `ConnectionWire`:
  ```json
  {
    "id": "wire_01",
    "source": { "node_id": "mic", "channel_index": 0 },
    "destination": { "node_id": "stream_out", "channel_index": 0 },
    "gain": 1.0,
    "muted": false
  }
  ```
- `VirtualDeviceDescriptor`:
  ```json
  {
    "id": "vdev-discord-aux-1",
    "name": "Discord Aux",
    "driver_type": "SoftwareLoopbackEndpoint",
    "render_endpoint_id": "vdev-discord-aux-1-sink",
    "capture_endpoint_id": "vdev-discord-aux-1-source",
    "channels": 2,
    "sample_rate": 48000,
    "is_active": true
  }
  ```

## 6. Troubleshooting & Edge Cases
- **Buffer Full**: `push_samples` returns the count of samples actually written without blocking. If count < slice length, audio capture overflow occurred. The caller should increment an underrun/overflow counter and avoid sleeping or blocking the audio thread.
- **Buffer Empty**: `pop_samples` returns the count of samples read. If insufficient samples are available, fill remaining frames with silence (0.0) to prevent audio stutter.
- **Digital Clipping Prevention**: When multiple loud audio sources sum above `1.0` (0 dBFS), the routing matrix's soft limiter applies hyperbolic tangent (`tanh`) compression, preventing harsh digital square-wave clipping while preserving dynamics below full scale.
- **Orphaned Wires on Node Removal**: Calling `RoutingMatrix::remove_node()` automatically deletes all wires connected to the target node as either source or destination, preventing stale or dangling pointer bugs in graph routing.
- **Missing Source Inputs**: If a source node has no active chunk submitted during `process_frame()`, it is treated as digital silence (0.0) without interrupting other summed signals.
