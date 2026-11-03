//! @file mod.rs
//! @brief Core audio engine module exposing ring buffers, chunk types, device enumeration,
//! routing matrix, virtual device adapters, and mixer engine.
//! @details Re-exports audio primitives, sample rate and channel abstractions, lock-free
//! SPSC ring buffer structures used for low-latency WASAPI routing, CPAL hardware endpoint
//! discovery, WASAPI process audio session scanners, pin routing matrices, virtual audio device
//! cable managers, and real-time mixer pipelines.

pub mod capture;
pub mod devices;
pub mod dsp;
pub mod matrix;
pub mod mixer;
pub mod process_scanner;
pub mod ring_buffer;
pub mod types;
pub mod virtual_device;

pub use capture::{spawn_microphone_capture_thread, LiveMeterTracker};
pub use devices::{list_audio_devices, AudioDeviceDescriptor, AudioDeviceType};
pub use dsp::{AutoDucker, NoiseGate};
pub use matrix::{AudioPinId, ConnectionWire, MixerNode, RoutingMatrix};
pub use mixer::AudioMixerEngine;
pub use process_scanner::{
    is_audio_candidate_application, list_audio_processes, AudioProcessDescriptor,
};
pub use ring_buffer::{
    AudioConsumer, AudioProducer, AudioRingBuffer, ChunkConsumer, ChunkProducer, ChunkRingBuffer,
};
pub use types::{
    calculate_peak_db, AudioChunk, ChannelCount, SampleRate, MAX_PEAK_DB, MIN_PEAK_DB,
};
pub use virtual_device::{
    VirtualCableType, VirtualDeviceDescriptor, VirtualDeviceManager,
};
