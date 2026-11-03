//! @file matrix.rs
//! @brief Audio routing matrix and multichannel pin patchbay.
//! @details Manages mixer nodes, virtual connection wires, channel pin routing,
//! linear volume attenuation, muting controls, and real-time mixing with smooth
//! hyperbolic tangent (`tanh`) soft-limiter saturation.

use super::types::AudioChunk;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// @brief Represents a specific audio channel pin on a mixer node.
/// @details Pins are identified by node ID and zero-based channel index.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AudioPinId {
    /// @brief Identifier of the mixer node owning this pin.
    pub node_id: String,
    /// @brief Zero-based channel index (0 for Left, 1 for Right, etc.).
    pub channel_index: u16,
}

impl AudioPinId {
    /// @brief Constructs a new AudioPinId.
    /// @param node_id Identifier of the owning mixer node.
    /// @param channel_index Zero-based channel index on the node.
    /// @return A new AudioPinId instance.
    pub fn new(node_id: impl Into<String>, channel_index: u16) -> Self {
        Self { node_id: node_id.into(), channel_index }
    }

    /// @brief Convenience constructor for Left / Mono pin (channel 0).
    /// @param node_id Identifier of the owning mixer node.
    /// @return AudioPinId for channel 0.
    pub fn left(node_id: impl Into<String>) -> Self {
        Self::new(node_id, 0)
    }

    /// @brief Convenience constructor for Right pin (channel 1).
    /// @param node_id Identifier of the owning mixer node.
    /// @return AudioPinId for channel 1.
    pub fn right(node_id: impl Into<String>) -> Self {
        Self::new(node_id, 1)
    }
}

/// @brief Virtual connection wire routing audio from a source pin to a destination pin.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConnectionWire {
    /// @brief Unique identifier of this connection wire.
    pub id: String,
    /// @brief Source pin delivering audio samples.
    pub source: AudioPinId,
    /// @brief Destination pin receiving and summing audio samples.
    pub destination: AudioPinId,
    /// @brief Linear gain scale applied across this wire (1.0 = unity gain).
    pub gain: f32,
    /// @brief Whether this connection wire is currently muted.
    pub muted: bool,
    /// @brief Whether to invert the phase of audio samples passing through this wire (-1.0x).
    pub phase_invert: bool,
    /// @brief Whether this wire serves as a monitoring tap to a personal listening endpoint.
    pub is_monitor: bool,
}

impl ConnectionWire {
    /// @brief Constructs a new active connection wire with specified gain.
    /// @param id Unique wire identifier.
    /// @param source Source pin delivering audio.
    /// @param destination Destination pin receiving audio.
    /// @param gain Linear gain multiplier.
    /// @return A new ConnectionWire instance.
    pub fn new(id: impl Into<String>, source: AudioPinId, destination: AudioPinId, gain: f32) -> Self {
        Self {
            id: id.into(),
            source,
            destination,
            gain,
            muted: false,
            phase_invert: false,
            is_monitor: false,
        }
    }

    /// @brief Builder method to set the mute state of this connection wire.
    /// @param muted Flag indicating whether the wire should be muted.
    /// @return Modified ConnectionWire instance.
    pub fn muted(mut self, muted: bool) -> Self {
        self.muted = muted;
        self
    }

    /// @brief Builder method to configure phase inversion on this connection wire.
    /// @param invert Flag indicating whether to invert audio phase.
    /// @return Modified ConnectionWire instance.
    pub fn with_phase_invert(mut self, invert: bool) -> Self {
        self.phase_invert = invert;
        self
    }

    /// @brief Builder method to configure monitor tap status on this connection wire.
    /// @param is_monitor Flag indicating whether wire is a monitor tap.
    /// @return Modified ConnectionWire instance.
    pub fn with_monitor(mut self, is_monitor: bool) -> Self {
        self.is_monitor = is_monitor;
        self
    }
}

/// @brief Represents a node in the audio routing matrix.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MixerNode {
    /// @brief Unique identifier of this mixer node.
    pub id: String,
    /// @brief Human-readable display name.
    pub name: String,
    /// @brief Linear volume gain applied to this node (1.0 = 0 dBFS unity gain).
    pub volume: f32,
    /// @brief Whether this node is muted.
    pub muted: bool,
    /// @brief Number of input audio channels supported.
    pub input_channels: u16,
    /// @brief Number of output audio channels supported.
    pub output_channels: u16,
    /// @brief Whether this node represents a personal listening monitor endpoint.
    pub is_monitor: bool,
    /// @brief Whether this monitor node suppresses original local audio when monitoring is active.
    pub mute_when_monitoring: bool,
    /// @brief Whether this node's underlying hardware endpoint is disconnected or offline.
    pub is_offline: bool,
    /// @brief Whether noise gate DSP filter is active for this node.
    pub noise_gate_enabled: bool,
    /// @brief Noise gate threshold in dBFS (defaults to -45.0).
    pub noise_gate_threshold_db: f32,
}

impl MixerNode {
    /// @brief Constructs a new MixerNode with unity volume (1.0) and unmuted state.
    /// @param id Unique identifier of the mixer node.
    /// @param name Human-readable display label.
    /// @param input_channels Number of input channels.
    /// @param output_channels Number of output channels.
    /// @return A new MixerNode instance.
    pub fn new(id: impl Into<String>, name: impl Into<String>, input_channels: u16, output_channels: u16) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            volume: 1.0,
            muted: false,
            input_channels,
            output_channels,
            is_monitor: false,
            mute_when_monitoring: false,
            is_offline: false,
            noise_gate_enabled: false,
            noise_gate_threshold_db: -45.0,
        }
    }

    /// @brief Builder method to configure node linear volume gain.
    /// @param volume Linear gain factor (e.g. 1.0 = unity, 0.5 = -6 dBFS).
    /// @return Modified MixerNode instance.
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume;
        self
    }

    /// @brief Builder method to configure node mute state.
    /// @param muted Flag indicating whether node should be muted.
    /// @return Modified MixerNode instance.
    pub fn with_muted(mut self, muted: bool) -> Self {
        self.muted = muted;
        self
    }

    /// @brief Builder method to configure monitor status for this node.
    /// @param is_monitor Flag indicating if node is a personal monitor endpoint.
    /// @return Modified MixerNode instance.
    pub fn with_monitor(mut self, is_monitor: bool) -> Self {
        self.is_monitor = is_monitor;
        self
    }

    /// @brief Builder method to configure mute-when-monitoring behavior.
    /// @param mute_when_monitoring Flag indicating whether to mute original stream during monitoring.
    /// @return Modified MixerNode instance.
    pub fn with_mute_when_monitoring(mut self, mute_when_monitoring: bool) -> Self {
        self.mute_when_monitoring = mute_when_monitoring;
        self
    }

    /// @brief Builder method to configure offline status for disconnected endpoints.
    /// @param is_offline Flag indicating if endpoint hardware is offline.
    /// @return Modified MixerNode instance.
    pub fn with_offline(mut self, is_offline: bool) -> Self {
        self.is_offline = is_offline;
        self
    }

    /// @brief Builder method to configure noise gate filtering for this node.
    /// @param enabled Flag indicating whether the noise gate is active.
    /// @param threshold_db Noise gate threshold in dBFS.
    /// @return Modified MixerNode instance.
    pub fn with_noise_gate(mut self, enabled: bool, threshold_db: f32) -> Self {
        self.noise_gate_enabled = enabled;
        self.noise_gate_threshold_db = threshold_db;
        self
    }
}

/// @brief Multi-channel audio routing matrix and sample mixing engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingMatrix {
    /// @brief Active mixer nodes keyed by unique node ID.
    nodes: HashMap<String, MixerNode>,
    /// @brief Active connection wires keyed by unique wire ID.
    wires: HashMap<String, ConnectionWire>,
    /// @brief Controls whether hyperbolic tangent soft-limiting is applied when samples exceed ±1.0.
    soft_limiter: bool,
    /// @brief Optional fallback destination node ID when a target endpoint is disconnected.
    fallback_node_id: Option<String>,
}

impl Default for RoutingMatrix {
    /// @brief Constructs a default RoutingMatrix with soft-limiting enabled.
    /// @return Default RoutingMatrix instance.
    fn default() -> Self {
        Self::new()
    }
}

impl RoutingMatrix {
    /// @brief Creates an empty RoutingMatrix with soft-limiting enabled by default.
    /// @return A new empty RoutingMatrix instance.
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            wires: HashMap::new(),
            soft_limiter: true,
            fallback_node_id: None,
        }
    }

    /// @brief Configures the optional fallback output destination node ID.
    /// @param node_id Fallback node ID or None to disable.
    pub fn set_fallback_node_id(&mut self, node_id: Option<String>) {
        self.fallback_node_id = node_id;
    }

    /// @brief Returns the configured fallback output destination node ID if any.
    /// @return Optional string slice with fallback node ID.
    pub fn fallback_node_id(&self) -> Option<&str> {
        self.fallback_node_id.as_deref()
    }

    /// @brief Configures whether a specific node is offline / disconnected from hardware.
    /// @param node_id Unique identifier of the target node.
    /// @param offline Whether the endpoint hardware is offline.
    pub fn set_node_offline(&mut self, node_id: &str, offline: bool) {
        if let Some(node) = self.nodes.get_mut(node_id) {
            node.is_offline = offline;
        }
    }

    /// @brief Registers a mixer node in the routing matrix.
    /// @param node The MixerNode instance to add.
    pub fn add_node(&mut self, node: MixerNode) {
        self.nodes.insert(node.id.clone(), node);
    }

    /// @brief Removes a mixer node and automatically disconnects all connected wires.
    /// @param node_id Identifier of the node to remove.
    /// @return The removed MixerNode if found, or None.
    pub fn remove_node(&mut self, node_id: &str) -> Option<MixerNode> {
        let removed = self.nodes.remove(node_id);
        if removed.is_some() {
            self.wires.retain(|_, w| w.source.node_id != node_id && w.destination.node_id != node_id);
        }
        removed
    }

    /// @brief Returns an immutable reference to a registered mixer node.
    /// @param node_id Identifier of the requested node.
    /// @return Option containing reference to MixerNode if present.
    pub fn get_node(&self, node_id: &str) -> Option<&MixerNode> {
        self.nodes.get(node_id)
    }

    /// @brief Returns a mutable reference to a registered mixer node.
    /// @param node_id Identifier of the requested node.
    /// @return Option containing mutable reference to MixerNode if present.
    pub fn get_node_mut(&mut self, node_id: &str) -> Option<&mut MixerNode> {
        self.nodes.get_mut(node_id)
    }

    /// @brief Returns an immutable map of all registered mixer nodes.
    /// @return Reference to HashMap containing nodes.
    pub fn nodes(&self) -> &HashMap<String, MixerNode> {
        &self.nodes
    }

    /// @brief Returns a mutable map of all registered mixer nodes.
    /// @return Mutable reference to HashMap containing nodes.
    pub fn nodes_mut(&mut self) -> &mut HashMap<String, MixerNode> {
        &mut self.nodes
    }

    /// @brief Connects two pins by registering a connection wire.
    /// @param wire ConnectionWire instance to insert or update.
    pub fn connect(&mut self, wire: ConnectionWire) {
        self.wires.insert(wire.id.clone(), wire);
    }

    /// @brief Disconnects a wire by its unique ID.
    /// @param wire_id Identifier of the wire to remove.
    /// @return The removed ConnectionWire if found, or None.
    pub fn disconnect(&mut self, wire_id: &str) -> Option<ConnectionWire> {
        self.wires.remove(wire_id)
    }

    /// @brief Returns an immutable reference to an active connection wire.
    /// @param wire_id Identifier of the requested wire.
    /// @return Option containing reference to ConnectionWire if present.
    pub fn get_wire(&self, wire_id: &str) -> Option<&ConnectionWire> {
        self.wires.get(wire_id)
    }

    /// @brief Returns a mutable reference to an active connection wire.
    /// @param wire_id Identifier of the requested wire.
    /// @return Option containing mutable reference to ConnectionWire if present.
    pub fn get_wire_mut(&mut self, wire_id: &str) -> Option<&mut ConnectionWire> {
        self.wires.get_mut(wire_id)
    }

    /// @brief Returns an immutable map of all active connection wires.
    /// @return Reference to HashMap containing wires.
    pub fn wires(&self) -> &HashMap<String, ConnectionWire> {
        &self.wires
    }

    /// @brief Enables or disables soft-limiter saturation when summing audio streams.
    /// @param enabled Flag indicating whether soft-limiting should be active.
    pub fn set_soft_limiter(&mut self, enabled: bool) {
        self.soft_limiter = enabled;
    }

    /// @brief Returns whether the soft limiter is currently enabled.
    /// @return `true` if soft limiter is enabled, `false` otherwise.
    pub fn is_soft_limiter_enabled(&self) -> bool {
        self.soft_limiter
    }

    /// @brief Renders and routes an audio frame chunk for a specific destination node.
    /// @details Evaluates all incoming wires terminating at the specified destination node.
    /// Performs channel mapping (1-to-1, stereo downmixing, mono splitting), scales by wire gain
    /// and source/destination volume, checks mute states, sums contributions per destination pin,
    /// and smoothly limits values exceeding ±1.0 amplitude via `tanh` to prevent harsh digital clipping.
    /// @param inputs Map of available source audio chunks keyed by source node ID.
    /// @param destination_node_id Node ID for which to render output audio.
    /// @return Rendered AudioChunk for the destination node.
    pub fn process_frame(&self, inputs: &HashMap<String, AudioChunk>, destination_node_id: &str) -> AudioChunk {
        let dest_node = match self.nodes.get(destination_node_id) {
            Some(node) => node,
            None => return AudioChunk::silence(0, 2, 48000),
        };

        let dest_channels = if dest_node.output_channels > 0 {
            dest_node.output_channels
        } else if dest_node.input_channels > 0 {
            dest_node.input_channels
        } else {
            2
        };

        if dest_node.is_offline {
            return AudioChunk::silence(0, dest_channels, 48000);
        }

        let is_fallback_target = self.fallback_node_id.as_deref() == Some(destination_node_id);
        let incoming_wires: Vec<&ConnectionWire> = self
            .wires
            .values()
            .filter(|wire| {
                if wire.muted {
                    return false;
                }
                if wire.destination.node_id == destination_node_id {
                    return true;
                }
                if is_fallback_target {
                    if let Some(target) = self.nodes.get(&wire.destination.node_id) {
                        return target.is_offline;
                    }
                }
                false
            })
            .collect();

        let mut target_frame_count = 0usize;
        let mut target_sample_rate = 48000u32;

        for wire in &incoming_wires {
            if let Some(chunk) = inputs.get(&wire.source.node_id) {
                target_frame_count = target_frame_count.max(chunk.frame_count());
                target_sample_rate = chunk.sample_rate;
            }
        }

        if target_frame_count == 0 {
            if let Some(first_chunk) = inputs.values().next() {
                target_frame_count = first_chunk.frame_count();
                target_sample_rate = first_chunk.sample_rate;
            }
        }

        if target_frame_count == 0 {
            return AudioChunk::silence(0, dest_channels, target_sample_rate);
        }

        if dest_node.muted || dest_node.volume == 0.0 {
            return AudioChunk::silence(target_frame_count, dest_channels, target_sample_rate);
        }

        let total_samples = target_frame_count.saturating_mul(dest_channels as usize);
        let mut output_samples = vec![0.0f32; total_samples];

        for frame_idx in 0..target_frame_count {
            for dst_ch in 0..dest_channels {
                let out_idx = frame_idx * (dest_channels as usize) + (dst_ch as usize);
                let mut sum = 0.0f32;

                for wire in &incoming_wires {
                    if wire.destination.channel_index != dst_ch {
                        continue;
                    }

                    let src_node = self.nodes.get(&wire.source.node_id);
                    if let Some(src) = src_node {
                        if src.muted {
                            continue;
                        }
                    }

                    let src_chunk = match inputs.get(&wire.source.node_id) {
                        Some(chunk) => chunk,
                        None => continue,
                    };

                    let src_ch_idx = wire.source.channel_index as usize;
                    let src_channels = src_chunk.channels as usize;

                    if src_ch_idx < src_channels && frame_idx < src_chunk.frame_count() {
                        let sample_idx = frame_idx * src_channels + src_ch_idx;
                        let raw_sample = src_chunk.samples[sample_idx];
                        let src_vol = src_node.map(|s| s.volume).unwrap_or(1.0);
                        let phase_scale = if wire.phase_invert { -1.0 } else { 1.0 };
                        sum += raw_sample * wire.gain * src_vol * phase_scale;
                    }
                }

                let scaled = sum * dest_node.volume;
                let final_val = if self.soft_limiter && !(-1.0..=1.0).contains(&scaled) {
                    scaled.tanh()
                } else {
                    scaled
                };

                output_samples[out_idx] = final_val;
            }
        }

        AudioChunk::new(output_samples, dest_channels, target_sample_rate)
    }
}
