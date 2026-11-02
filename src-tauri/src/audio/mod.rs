//! @file mod.rs
//! @brief Core audio engine module exposing ring buffers, chunk types, and DSP algorithms.
//! @details Re-exports audio primitives, sample rate and channel abstractions, and lock-free
//! SPSC ring buffer structures used for low-latency WASAPI routing.

pub mod ring_buffer;
pub mod types;

pub use ring_buffer::{AudioConsumer, AudioProducer, AudioRingBuffer, ChunkConsumer, ChunkProducer, ChunkRingBuffer};
pub use types::{calculate_peak_db, AudioChunk, ChannelCount, SampleRate, MAX_PEAK_DB, MIN_PEAK_DB};
