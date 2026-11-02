//! @file mixer.rs
//! @brief Real-time audio mixer engine managing inputs, matrix routing, and rendering endpoints.
//! @details Provides `AudioMixerEngine` to ingest audio chunks from application captures
//! and hardware inputs, execute routing matrix graph transformations, and produce output
//! chunks for audio render devices.

use super::matrix::{ConnectionWire, MixerNode, RoutingMatrix};
use super::types::AudioChunk;
use std::collections::HashMap;

/// @brief Real-time audio mixer pipeline engine.
/// @details Combines a `RoutingMatrix` graph with active input staging buffers.
/// Coordinates multi-source ingestion, gain and muting application, soft saturation,
/// and output rendering for destination endpoints.
#[derive(Debug, Clone)]
pub struct AudioMixerEngine {
    /// @brief Audio routing matrix managing nodes and pin connections.
    matrix: RoutingMatrix,
    /// @brief Staging input chunks indexed by source node identifier.
    inputs: HashMap<String, AudioChunk>,
    /// @brief Nominal sample rate in Hertz (e.g. 48000).
    sample_rate: u32,
}

impl AudioMixerEngine {
    /// @brief Constructs a new AudioMixerEngine with a given nominal sample rate.
    /// @param sample_rate Nominal sample rate in Hertz (e.g. 44100, 48000).
    /// @return A new AudioMixerEngine instance.
    pub fn new(sample_rate: u32) -> Self {
        Self {
            matrix: RoutingMatrix::new(),
            inputs: HashMap::new(),
            sample_rate,
        }
    }

    /// @brief Returns the nominal sample rate of the mixer engine.
    /// @return Sample rate in Hertz.
    #[inline]
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// @brief Configures the nominal sample rate of the mixer engine.
    /// @param sample_rate Sample rate in Hertz.
    #[inline]
    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate;
    }

    /// @brief Returns an immutable reference to the underlying routing matrix.
    /// @return Reference to RoutingMatrix.
    #[inline]
    pub fn matrix(&self) -> &RoutingMatrix {
        &self.matrix
    }

    /// @brief Returns a mutable reference to the underlying routing matrix.
    /// @return Mutable reference to RoutingMatrix.
    #[inline]
    pub fn matrix_mut(&mut self) -> &mut RoutingMatrix {
        &mut self.matrix
    }

    /// @brief Adds a mixer node into the underlying routing matrix.
    /// @param node MixerNode to add.
    #[inline]
    pub fn add_node(&mut self, node: MixerNode) {
        self.matrix.add_node(node);
    }

    /// @brief Removes a mixer node from the underlying routing matrix.
    /// @param node_id Identifier of the node to remove.
    /// @return The removed MixerNode if found, or None.
    #[inline]
    pub fn remove_node(&mut self, node_id: &str) -> Option<MixerNode> {
        self.inputs.remove(node_id);
        self.matrix.remove_node(node_id)
    }

    /// @brief Connects two pins in the underlying routing matrix.
    /// @param wire ConnectionWire to register.
    #[inline]
    pub fn connect(&mut self, wire: ConnectionWire) {
        self.matrix.connect(wire);
    }

    /// @brief Disconnects a wire in the underlying routing matrix.
    /// @param wire_id Identifier of the wire to remove.
    /// @return The removed ConnectionWire if found, or None.
    #[inline]
    pub fn disconnect(&mut self, wire_id: &str) -> Option<ConnectionWire> {
        self.matrix.disconnect(wire_id)
    }

    /// @brief Submits an input audio chunk for a specific source node.
    /// @param node_id Identifier of the source node.
    /// @param chunk Contiguous audio chunk to stage for mixing.
    pub fn submit_input(&mut self, node_id: impl Into<String>, chunk: AudioChunk) {
        self.inputs.insert(node_id.into(), chunk);
    }

    /// @brief Clears all staged input chunks.
    pub fn clear_inputs(&mut self) {
        self.inputs.clear();
    }

    /// @brief Renders the mixed audio chunk for a given destination node ID.
    /// @param destination_node_id Identifier of the destination node to process.
    /// @return Rendered AudioChunk computed from active staged inputs.
    pub fn process_destination(&mut self, destination_node_id: &str) -> AudioChunk {
        self.matrix.process_frame(&self.inputs, destination_node_id)
    }

    /// @brief Renders mixed output audio chunks for all registered output nodes.
    /// @details Evaluates all mixer nodes configured with `output_channels > 0`.
    /// @return Map of rendered AudioChunks keyed by destination node ID.
    pub fn render_all(&self) -> HashMap<String, AudioChunk> {
        let mut results = HashMap::new();
        for (node_id, node) in self.matrix.nodes() {
            if node.output_channels > 0 {
                let chunk = self.matrix.process_frame(&self.inputs, node_id);
                results.insert(node_id.clone(), chunk);
            }
        }
        results
    }
}
