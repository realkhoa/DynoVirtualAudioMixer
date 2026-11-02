//! @file devices.rs
//! @brief Audio device enumeration and hardware capability querying using CPAL and WASAPI.
//! @details Discovers physical and virtual audio inputs and outputs on the host system,
//! categorizing devices by type, default status, channel configuration, and supported sample rates.

use cpal::traits::{DeviceTrait, HostTrait};
use serde::{Deserialize, Serialize};

/// @brief Classification of audio endpoint devices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AudioDeviceType {
    /// @brief Audio input device (e.g., microphone, line-in).
    Input,
    /// @brief Audio output device (e.g., headphones, speakers).
    Output,
    /// @brief Virtual audio device (e.g., Dyno virtual cable, software bus).
    Virtual,
}

impl AudioDeviceType {
    /// @brief Checks whether the device is an input capture endpoint.
    /// @return `true` if Input, `false` otherwise.
    #[inline]
    pub fn is_input(&self) -> bool {
        matches!(self, AudioDeviceType::Input)
    }

    /// @brief Checks whether the device is an output render endpoint.
    /// @return `true` if Output, `false` otherwise.
    #[inline]
    pub fn is_output(&self) -> bool {
        matches!(self, AudioDeviceType::Output)
    }

    /// @brief Checks whether the device is a virtual loopback or software cable endpoint.
    /// @return `true` if Virtual, `false` otherwise.
    #[inline]
    pub fn is_virtual(&self) -> bool {
        matches!(self, AudioDeviceType::Virtual)
    }
}

/// @brief Metadata descriptor representing a physical or virtual audio endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AudioDeviceDescriptor {
    /// @brief Unique hardware or system identifier for the audio endpoint.
    pub id: String,
    /// @brief Human-readable friendly display name.
    pub name: String,
    /// @brief Classification of device (Input, Output, Virtual).
    pub device_type: AudioDeviceType,
    /// @brief Number of supported audio channels (e.g., 1 for mono, 2 for stereo).
    pub channels: u16,
    /// @brief Native or default sample rate in Hertz (e.g., 44100, 48000).
    pub sample_rate: u32,
    /// @brief Whether this endpoint is currently the system default for its role.
    pub is_default: bool,
}

impl AudioDeviceDescriptor {
    /// @brief Constructs a new AudioDeviceDescriptor instance.
    /// @param id Unique device identifier.
    /// @param name Friendly display name of the device.
    /// @param device_type Classification category (Input, Output, Virtual).
    /// @param channels Number of audio channels.
    /// @param sample_rate Sample rate in Hertz.
    /// @param is_default Flag indicating if this is the default audio endpoint.
    /// @return Newly initialized AudioDeviceDescriptor.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        device_type: AudioDeviceType,
        channels: u16,
        sample_rate: u32,
        is_default: bool,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            device_type,
            channels,
            sample_rate,
            is_default,
        }
    }

    /// @brief Determines if the descriptor represents an input endpoint.
    /// @return `true` if Input, `false` otherwise.
    #[inline]
    pub fn is_input(&self) -> bool {
        self.device_type.is_input()
    }

    /// @brief Determines if the descriptor represents an output endpoint.
    /// @return `true` if Output, `false` otherwise.
    #[inline]
    pub fn is_output(&self) -> bool {
        self.device_type.is_output()
    }

    /// @brief Determines if the descriptor represents a virtual endpoint.
    /// @return `true` if Virtual, `false` otherwise.
    #[inline]
    pub fn is_virtual(&self) -> bool {
        self.device_type.is_virtual()
    }
}

/// @brief Helper to generate a sanitized identifier slug from a device name.
/// @param name Original device name string.
/// @return Lowercased alphanumeric slug with hyphens.
fn slugify(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// @brief Enumerates all physical and virtual audio devices available on the host machine.
/// @details Queries the system audio host (WASAPI on Windows, CoreAudio on macOS, ALSA on Linux)
/// using CPAL. Identifies default input and output endpoints, retrieves native channel
/// counts and sample rates, and handles headless or missing hardware environments gracefully.
/// @return `Ok(Vec<AudioDeviceDescriptor>)` containing discovered endpoints, or `Err(String)` on failure.
pub fn list_audio_devices() -> Result<Vec<AudioDeviceDescriptor>, String> {
    let host = cpal::default_host();
    let mut descriptors = Vec::new();

    // Query default device names for matching
    let default_input_name = host.default_input_device().and_then(|d| d.name().ok());
    let default_output_name = host.default_output_device().and_then(|d| d.name().ok());

    // Enumerate physical and virtual input capture devices
    if let Ok(input_devices) = host.input_devices() {
        for (index, device) in input_devices.enumerate() {
            if let Ok(name) = device.name() {
                let is_default = default_input_name.as_deref() == Some(&name);

                // Attempt to retrieve default config, falling back to supported ranges or stereo 48kHz
                let (channels, sample_rate) = device
                    .default_input_config()
                    .map(|cfg| (cfg.channels(), cfg.sample_rate().0))
                    .ok()
                    .or_else(|| {
                        device
                            .supported_input_configs()
                            .ok()
                            .and_then(|mut iter| iter.next())
                            .map(|cfg| (cfg.channels(), cfg.max_sample_rate().0))
                    })
                    .unwrap_or((2, 48000));

                let is_virtual = name.to_lowercase().contains("dyno virtual");
                let device_type = if is_virtual {
                    AudioDeviceType::Virtual
                } else {
                    AudioDeviceType::Input
                };

                let slug = slugify(&name);
                let id = format!("in-{}-{}", index, if slug.is_empty() { "endpoint" } else { &slug });

                descriptors.push(AudioDeviceDescriptor {
                    id,
                    name,
                    device_type,
                    channels,
                    sample_rate,
                    is_default,
                });
            }
        }
    }

    // Enumerate physical and virtual output playback devices
    if let Ok(output_devices) = host.output_devices() {
        for (index, device) in output_devices.enumerate() {
            if let Ok(name) = device.name() {
                let is_default = default_output_name.as_deref() == Some(&name);

                // Attempt to retrieve default config, falling back to supported ranges or stereo 48kHz
                let (channels, sample_rate) = device
                    .default_output_config()
                    .map(|cfg| (cfg.channels(), cfg.sample_rate().0))
                    .ok()
                    .or_else(|| {
                        device
                            .supported_output_configs()
                            .ok()
                            .and_then(|mut iter| iter.next())
                            .map(|cfg| (cfg.channels(), cfg.max_sample_rate().0))
                    })
                    .unwrap_or((2, 48000));

                let is_virtual = name.to_lowercase().contains("dyno virtual");
                let device_type = if is_virtual {
                    AudioDeviceType::Virtual
                } else {
                    AudioDeviceType::Output
                };

                let slug = slugify(&name);
                let id = format!("out-{}-{}", index, if slug.is_empty() { "endpoint" } else { &slug });

                descriptors.push(AudioDeviceDescriptor {
                    id,
                    name,
                    device_type,
                    channels,
                    sample_rate,
                    is_default,
                });
            }
        }
    }

    Ok(descriptors)
}
