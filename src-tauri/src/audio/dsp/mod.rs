//! @file mod.rs
//! @brief Digital signal processing (DSP) filters and audio dynamics processors.
//! @details Houses low-latency, real-time audio processors including noise gates
//! for microphone background attenuation and sidechain auto-duckers for streamer voice priority.

pub mod ducker;
pub mod gate;

pub use ducker::AutoDucker;
pub use gate::NoiseGate;
