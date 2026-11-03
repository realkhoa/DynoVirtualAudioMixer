//! @file dto.rs
//! @brief Data transfer objects for audio routing nodes, connection wires, and real-time meters.
//! @details Facilitates strongly-typed serialization and deserialization between the Tauri IPC
//! bridge and the frontend React Flow audio patchbay canvas.

use crate::audio::matrix::{AudioPinId, ConnectionWire, MixerNode, RoutingMatrix};
use serde::{Deserialize, Serialize};

/// @brief Returns default linear volume gain (1.0).
/// @return Default volume multiplier as `1.0f32`.
fn default_volume() -> f32 {
    1.0
}

/// @brief Returns default connection wire gain (1.0).
/// @return Default gain multiplier as `1.0f32`.
fn default_gain() -> f32 {
    1.0
}

/// @brief Returns default noise gate threshold (-45.0 dBFS).
/// @return Default noise gate threshold as `-45.0f32`.
fn default_noise_gate_threshold() -> f32 {
    -45.0
}

/// @brief Data transfer object representing a mixer node in the audio routing graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeDto {
    /// @brief Unique identifier of the mixer node.
    pub id: String,
    /// @brief Human-readable display label.
    pub name: String,
    /// @brief Optional node category (e.g., "input", "output", "virtual", "app").
    #[serde(default)]
    pub node_type: Option<String>,
    /// @brief Linear volume gain applied to this node (1.0 = 0 dBFS unity gain).
    #[serde(default = "default_volume")]
    pub volume: f32,
    /// @brief Whether this node is muted.
    #[serde(default)]
    pub muted: bool,
    /// @brief Number of input audio channels supported.
    #[serde(default)]
    pub input_channels: u16,
    /// @brief Number of output audio channels supported.
    #[serde(default)]
    pub output_channels: u16,
    /// @brief Whether this node represents a personal listening monitor endpoint.
    #[serde(default)]
    pub is_monitor: bool,
    /// @brief Whether this monitor node suppresses original local audio when monitoring is active.
    #[serde(default)]
    pub mute_when_monitoring: bool,
    /// @brief Whether this node's underlying hardware endpoint is disconnected or offline.
    #[serde(default)]
    pub is_offline: bool,
    /// @brief Whether noise gate DSP filter is active for this node.
    #[serde(default)]
    pub noise_gate_enabled: bool,
    /// @brief Noise gate threshold in dBFS (defaults to -45.0).
    #[serde(default = "default_noise_gate_threshold")]
    pub noise_gate_threshold_db: f32,
}

impl NodeDto {
    /// @brief Constructs a new NodeDto instance with default unity volume and unmuted state.
    /// @param id Unique node identifier.
    /// @param name Friendly display label.
    /// @param input_channels Number of input audio channels.
    /// @param output_channels Number of output audio channels.
    /// @return Initialized NodeDto instance.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        input_channels: u16,
        output_channels: u16,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            node_type: None,
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
}

impl From<&MixerNode> for NodeDto {
    /// @brief Converts a reference to a core MixerNode into its corresponding DTO representation.
    /// @param node Reference to the core MixerNode.
    /// @return NodeDto instance.
    fn from(node: &MixerNode) -> Self {
        Self {
            id: node.id.clone(),
            name: node.name.clone(),
            node_type: None,
            volume: node.volume,
            muted: node.muted,
            input_channels: node.input_channels,
            output_channels: node.output_channels,
            is_monitor: node.is_monitor,
            mute_when_monitoring: node.mute_when_monitoring,
            is_offline: node.is_offline,
            noise_gate_enabled: node.noise_gate_enabled,
            noise_gate_threshold_db: node.noise_gate_threshold_db,
        }
    }
}

impl From<MixerNode> for NodeDto {
    /// @brief Converts an owned core MixerNode into its corresponding DTO representation.
    /// @param node Owned core MixerNode.
    /// @return NodeDto instance.
    fn from(node: MixerNode) -> Self {
        Self::from(&node)
    }
}

impl From<&NodeDto> for MixerNode {
    /// @brief Converts a reference to a NodeDto into its core MixerNode representation.
    /// @param dto Reference to the NodeDto.
    /// @return MixerNode instance.
    fn from(dto: &NodeDto) -> Self {
        MixerNode::new(&dto.id, &dto.name, dto.input_channels, dto.output_channels)
            .with_volume(dto.volume)
            .with_muted(dto.muted)
            .with_monitor(dto.is_monitor)
            .with_mute_when_monitoring(dto.mute_when_monitoring)
            .with_offline(dto.is_offline)
            .with_noise_gate(dto.noise_gate_enabled, dto.noise_gate_threshold_db)
    }
}

impl From<NodeDto> for MixerNode {
    /// @brief Converts an owned NodeDto into its core MixerNode representation.
    /// @param dto Owned NodeDto.
    /// @return MixerNode instance.
    fn from(dto: NodeDto) -> Self {
        MixerNode::from(&dto)
    }
}

/// @brief Data transfer object representing a connection wire between two channel pins.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConnectionDto {
    /// @brief Unique wire connection identifier.
    pub id: String,
    /// @brief Identifier of the source mixer node.
    #[serde(alias = "source", alias = "source_node")]
    pub source_node_id: String,
    /// @brief Zero-based channel index on the source node (e.g. 0 for Left, 1 for Right).
    #[serde(default, alias = "source_channel_index", alias = "src_ch")]
    pub source_channel: u16,
    /// @brief Identifier of the destination mixer node.
    #[serde(alias = "destination", alias = "target", alias = "destination_node")]
    pub destination_node_id: String,
    /// @brief Zero-based channel index on the destination node.
    #[serde(
        default,
        alias = "destination_channel_index",
        alias = "dst_ch",
        alias = "target_channel"
    )]
    pub destination_channel: u16,
    /// @brief Linear gain scale applied across this wire (1.0 = unity gain).
    #[serde(default = "default_gain")]
    pub gain: f32,
    /// @brief Whether this connection wire is currently muted.
    #[serde(default)]
    pub muted: bool,
    /// @brief Whether to invert the phase of audio samples passing through this wire.
    #[serde(default)]
    pub phase_invert: bool,
    /// @brief Whether this wire is configured as a monitoring tap.
    #[serde(default)]
    pub is_monitor: bool,
}

impl ConnectionDto {
    /// @brief Constructs a new ConnectionDto instance.
    /// @param id Unique wire identifier.
    /// @param source_node_id Identifier of the source mixer node.
    /// @param source_channel Zero-based source channel pin index.
    /// @param destination_node_id Identifier of the destination mixer node.
    /// @param destination_channel Zero-based destination channel pin index.
    /// @param gain Linear gain scale multiplier.
    /// @return Initialized ConnectionDto instance.
    pub fn new(
        id: impl Into<String>,
        source_node_id: impl Into<String>,
        source_channel: u16,
        destination_node_id: impl Into<String>,
        destination_channel: u16,
        gain: f32,
    ) -> Self {
        Self {
            id: id.into(),
            source_node_id: source_node_id.into(),
            source_channel,
            destination_node_id: destination_node_id.into(),
            destination_channel,
            gain,
            muted: false,
            phase_invert: false,
            is_monitor: false,
        }
    }
}

impl From<&ConnectionWire> for ConnectionDto {
    /// @brief Converts a reference to a core ConnectionWire into its DTO representation.
    /// @param wire Reference to the core ConnectionWire.
    /// @return ConnectionDto instance.
    fn from(wire: &ConnectionWire) -> Self {
        Self {
            id: wire.id.clone(),
            source_node_id: wire.source.node_id.clone(),
            source_channel: wire.source.channel_index,
            destination_node_id: wire.destination.node_id.clone(),
            destination_channel: wire.destination.channel_index,
            gain: wire.gain,
            muted: wire.muted,
            phase_invert: wire.phase_invert,
            is_monitor: wire.is_monitor,
        }
    }
}

impl From<ConnectionWire> for ConnectionDto {
    /// @brief Converts an owned ConnectionWire into its DTO representation.
    /// @param wire Owned ConnectionWire.
    /// @return ConnectionDto instance.
    fn from(wire: ConnectionWire) -> Self {
        Self::from(&wire)
    }
}

impl From<&ConnectionDto> for ConnectionWire {
    /// @brief Converts a reference to a ConnectionDto into its core ConnectionWire representation.
    /// @param dto Reference to the ConnectionDto.
    /// @return ConnectionWire instance.
    fn from(dto: &ConnectionDto) -> Self {
        Self {
            id: dto.id.clone(),
            source: AudioPinId::new(&dto.source_node_id, dto.source_channel),
            destination: AudioPinId::new(&dto.destination_node_id, dto.destination_channel),
            gain: dto.gain,
            muted: dto.muted,
            phase_invert: dto.phase_invert,
            is_monitor: dto.is_monitor,
        }
    }
}

impl From<ConnectionDto> for ConnectionWire {
    /// @brief Converts an owned ConnectionDto into its core ConnectionWire representation.
    /// @param dto Owned ConnectionDto.
    /// @return ConnectionWire instance.
    fn from(dto: ConnectionDto) -> Self {
        ConnectionWire::from(&dto)
    }
}

/// @brief Data transfer object encapsulating the entire audio routing patchbay graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioGraphDto {
    /// @brief Collection of mixer nodes in the patchbay.
    pub nodes: Vec<NodeDto>,
    /// @brief Collection of patch connection wires linking channel pins.
    pub connections: Vec<ConnectionDto>,
}

impl AudioGraphDto {
    /// @brief Constructs a new AudioGraphDto instance.
    /// @param nodes List of node DTOs.
    /// @param connections List of connection DTOs.
    /// @return Initialized AudioGraphDto.
    pub fn new(nodes: Vec<NodeDto>, connections: Vec<ConnectionDto>) -> Self {
        Self { nodes, connections }
    }

    /// @brief Converts the DTO graph into a newly constructed core RoutingMatrix.
    /// @return RoutingMatrix containing all nodes and connection wires from this graph.
    pub fn to_routing_matrix(&self) -> RoutingMatrix {
        let mut matrix = RoutingMatrix::new();
        for node in &self.nodes {
            matrix.add_node(MixerNode::from(node));
        }
        for conn in &self.connections {
            matrix.connect(ConnectionWire::from(conn));
        }
        matrix
    }

    /// @brief Builds an AudioGraphDto snapshot from an existing core RoutingMatrix.
    /// @param matrix Reference to the active RoutingMatrix.
    /// @return AudioGraphDto representing the current matrix graph.
    pub fn from_routing_matrix(matrix: &RoutingMatrix) -> Self {
        let node_map = matrix.nodes();
        let mut nodes: Vec<NodeDto> = node_map.values().map(NodeDto::from).collect();
        nodes.sort_by(|a, b| a.id.cmp(&b.id));

        let wire_map = matrix.wires();
        let mut connections: Vec<ConnectionDto> = wire_map.values().map(ConnectionDto::from).collect();
        connections.sort_by(|a, b| a.id.cmp(&b.id));

        Self { nodes, connections }
    }
}

impl From<&RoutingMatrix> for AudioGraphDto {
    /// @brief Converts a reference to a RoutingMatrix into an AudioGraphDto.
    /// @param matrix Reference to the active RoutingMatrix.
    /// @return AudioGraphDto instance.
    fn from(matrix: &RoutingMatrix) -> Self {
        Self::from_routing_matrix(matrix)
    }
}

impl From<RoutingMatrix> for AudioGraphDto {
    /// @brief Converts an owned RoutingMatrix into an AudioGraphDto.
    /// @param matrix Owned RoutingMatrix.
    /// @return AudioGraphDto instance.
    fn from(matrix: RoutingMatrix) -> Self {
        Self::from_routing_matrix(&matrix)
    }
}

impl From<&AudioGraphDto> for RoutingMatrix {
    /// @brief Converts a reference to an AudioGraphDto into a core RoutingMatrix.
    /// @param graph Reference to AudioGraphDto.
    /// @return Newly populated RoutingMatrix instance.
    fn from(graph: &AudioGraphDto) -> Self {
        graph.to_routing_matrix()
    }
}

impl From<AudioGraphDto> for RoutingMatrix {
    /// @brief Converts an owned AudioGraphDto into a core RoutingMatrix.
    /// @param graph Owned AudioGraphDto.
    /// @return Newly populated RoutingMatrix instance.
    fn from(graph: AudioGraphDto) -> Self {
        graph.to_routing_matrix()
    }
}

/// @brief Data transfer object containing real-time peak decibel telemetry for a single mixer node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeMeterDto {
    /// @brief Unique identifier of the mixer node.
    pub node_id: String,
    /// @brief Left channel peak amplitude in decibels (clamped to [-60.0, 0.0] dBFS).
    pub peak_left: f32,
    /// @brief Right channel peak amplitude in decibels (clamped to [-60.0, 0.0] dBFS).
    pub peak_right: f32,
}

/// @brief Data transfer object emitted at 60 FPS containing peak decibel telemetry for all active nodes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeterUpdateDto {
    /// @brief Epoch timestamp in milliseconds when this meter sample was captured.
    pub timestamp_ms: u64,
    /// @brief Array of peak meter readings for active mixer nodes.
    pub meters: Vec<NodeMeterDto>,
}

impl MeterUpdateDto {
    /// @brief Constructs a new MeterUpdateDto instance.
    /// @param timestamp_ms Timestamp in milliseconds since Unix epoch.
    /// @param meters List of individual node meter readings.
    /// @return Initialized MeterUpdateDto.
    pub fn new(timestamp_ms: u64, meters: Vec<NodeMeterDto>) -> Self {
        Self {
            timestamp_ms,
            meters,
        }
    }
}
