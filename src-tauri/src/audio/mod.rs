//! @file mod.rs
//! @brief Core audio engine module exposing ring buffers, chunk types, device enumeration, and process audio detection.
//! @details Re-exports audio primitives, sample rate and channel abstractions, lock-free
//! SPSC ring buffer structures used for low-latency WASAPI routing, CPAL hardware endpoint
//! discovery, and WASAPI process audio session scanners.

pub mod devices;
pub mod process_scanner;
pub mod ring_buffer;
pub mod types;

pub use devices::{list_audio_devices, AudioDeviceDescriptor, AudioDeviceType};
pub use process_scanner::{
    is_audio_candidate_application, list_audio_processes, AudioProcessDescriptor,
};
pub use ring_buffer::{
    AudioConsumer, AudioProducer, AudioRingBuffer, ChunkConsumer, ChunkProducer, ChunkRingBuffer,
};
pub use types::{
    calculate_peak_db, AudioChunk, ChannelCount, SampleRate, MAX_PEAK_DB, MIN_PEAK_DB,
};
