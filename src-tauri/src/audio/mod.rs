//! @file mod.rs
//! @brief Core audio engine module exposing ring buffers, chunk types, device enumeration,
//! routing matrix, and mixer engine.
//! @details Re-exports audio primitives, sample rate and channel abstractions, lock-free
//! SPSC ring buffer structures used for low-latency WASAPI routing, CPAL hardware endpoint
//! discovery, WASAPI process audio session scanners, pin routing matrices, and real-time mixer pipelines.

pub mod devices;
pub mod matrix;
pub mod mixer;
pub mod process_scanner;
pub mod ring_buffer;
pub mod types;

pub use devices::{list_audio_devices, AudioDeviceDescriptor, AudioDeviceType};
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
