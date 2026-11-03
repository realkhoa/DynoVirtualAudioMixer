/**
 * @file audio.ts
 * @brief TypeScript domain models and schemas for audio patchbay nodes, devices, and telemetry.
 * @details Provides strongly typed contracts for WASAPI audio endpoints, running processes,
 * virtual cables, React Flow node data payloads, and 60 FPS peak decibel meter updates.
 */

/**
 * @brief Classification of physical and virtual audio endpoint devices.
 */
export type AudioDeviceType = "Input" | "Output" | "Virtual";

/**
 * @brief Metadata descriptor representing a physical or virtual audio endpoint.
 */
export interface AudioDeviceDescriptor {
  /** @brief Unique hardware or system identifier for the endpoint. */
  id: string;
  /** @brief Human-readable display label (e.g., "Realtek High Definition Audio"). */
  name: string;
  /** @brief Device classification category (Input, Output, or Virtual). */
  device_type: AudioDeviceType;
  /** @brief Number of supported audio channels (e.g., 2 for stereo). */
  channels: number;
  /** @brief Native sample rate in Hertz (e.g., 44100, 48000). */
  sample_rate: number;
  /** @brief Whether this endpoint is the system default for its role. */
  is_default: boolean;
}

/**
 * @brief Metadata descriptor representing an audio-producing process running on the host system.
 */
export interface AudioProcessDescriptor {
  /** @brief Operating system Process ID (PID). */
  pid: number;
  /** @brief Executable binary name (e.g., "spotify.exe", "discord.exe"). */
  name: string;
  /** @brief Main window or application title (e.g., "Spotify Free"). */
  title: string;
  /** @brief Whether the process currently holds an active WASAPI audio session. */
  has_audio_session: boolean;
}

/**
 * @brief Supported driver technologies for virtual audio cables.
 */
export type VirtualCableType =
  | "VBCable"
  | "VirtualAudioCable"
  | "SoftwareLoopbackEndpoint";

/**
 * @brief Metadata descriptor representing a virtual cable or software loopback endpoint.
 */
export interface VirtualDeviceDescriptor {
  /** @brief Unique virtual device identifier. */
  id: string;
  /** @brief User-friendly display label (e.g., "Stream Mix Bus"). */
  name: string;
  /** @brief Driver technology powering the endpoint. */
  driver_type: VirtualCableType;
  /** @brief System audio endpoint ID used for rendering into this cable. */
  render_endpoint_id?: string | null;
  /** @brief System audio endpoint ID used for capturing from this cable. */
  capture_endpoint_id?: string | null;
  /** @brief Number of audio channels supported. */
  channels: number;
  /** @brief Sample rate in Hertz. */
  sample_rate: number;
  /** @brief Whether the virtual device is active and accepting audio routing. */
  is_active: boolean;
}

/**
 * @brief Classification of patchbay canvas node categories.
 */
export type AudioNodeType = "source" | "output" | "virtual";

/**
 * @brief Icon classification for visually representing different audio sources and sinks.
 */
export type AudioIconType =
  | "app"
  | "mic"
  | "speaker"
  | "headphones"
  | "virtual"
  | "music"
  | "browser"
  | "game"
  | "default";

/**
 * @brief Data payload stored in React Flow nodes for audio patchbay routing.
 */
export interface AudioNodeData {
  /** @brief Unique node identifier in the patchbay matrix. */
  id: string;
  /** @brief Primary display label. */
  name: string;
  /** @brief Optional secondary descriptor or subtext (e.g., process PID or endpoint type). */
  subtext?: string;
  /** @brief Classification of the node (source, output, or virtual). */
  nodeType: AudioNodeType;
  /** @brief Optional OS Process ID for application audio nodes. */
  pid?: number;
  /** @brief Visual icon classification. */
  iconType?: AudioIconType;
  /** @brief Linear volume gain (0.0 to 1.5, where 1.0 represents unity 0 dB gain). */
  volume: number;
  /** @brief Whether this node is currently muted. */
  muted: boolean;
  /** @brief Number of input channels (pins on the left). */
  inputChannels: number;
  /** @brief Number of output channels (pins on the right). */
  outputChannels: number;
  /** @brief Left channel peak decibel level clamped between -60 dBFS and 0 dBFS. */
  leftLevelDb: number;
  /** @brief Right channel peak decibel level clamped between -60 dBFS and 0 dBFS. */
  rightLevelDb: number;
  /** @brief Whether this node serves as a personal listening monitor. */
  isMonitor?: boolean;
  /** @brief Whether to mute original local sound when monitoring is active. */
  muteWhenMonitoring?: boolean;
  /** @brief Whether the underlying hardware endpoint is currently disconnected. */
  isOffline?: boolean;
  /** @brief Whether the noise gate filter is enabled for this input. */
  noiseGateEnabled?: boolean;
  /** @brief Noise gate threshold in dBFS (e.g. -45 dBFS). */
  noiseGateThresholdDb?: number;
  /** @brief Optional callback fired when the volume slider is adjusted. */
  onVolumeChange?: (id: string, volume: number) => void;
  /** @brief Optional callback fired when the mute state is toggled. */
  onMuteToggle?: (id: string, muted: boolean) => void;
  /** @brief Optional callback fired when the node is deleted by the user. */
  onRemove?: (id: string) => void;
  /** @brief Index signature to satisfy Record<string, unknown> constraint of React Flow. */
  [key: string]: unknown;
}

/**
 * @brief Data transfer object for node configuration serialized to/from Tauri backend.
 */
export interface NodeDto {
  /** @brief Unique node identifier. */
  id: string;
  /** @brief Display label. */
  name: string;
  /** @brief Optional node type identifier. */
  node_type?: string;
  /** @brief Optional OS Process ID for application audio nodes. */
  pid?: number;
  /** @brief Linear volume multiplier. */
  volume: number;
  /** @brief Muted flag. */
  muted: boolean;
  /** @brief Input channel count. */
  input_channels: number;
  /** @brief Output channel count. */
  output_channels: number;
  /** @brief Whether this node serves as a personal listening monitor. */
  is_monitor?: boolean;
  /** @brief Whether to mute original stream when monitoring is active. */
  mute_when_monitoring?: boolean;
  /** @brief Whether the endpoint hardware is offline. */
  is_offline?: boolean;
  /** @brief Whether noise gate DSP filter is active. */
  noise_gate_enabled?: boolean;
  /** @brief Noise gate threshold in dBFS. */
  noise_gate_threshold_db?: number;
}

/**
 * @brief Data transfer object for a connection wire between two channel pins.
 */
export interface ConnectionDto {
  /** @brief Unique wire identifier. */
  id: string;
  /** @brief Source mixer node ID. */
  source_node_id: string;
  /** @brief Zero-based source channel pin index (0 = L, 1 = R). */
  source_channel: number;
  /** @brief Destination mixer node ID. */
  destination_node_id: string;
  /** @brief Zero-based destination channel pin index (0 = L, 1 = R). */
  destination_channel: number;
  /** @brief Linear gain scale applied across the wire. */
  gain: number;
  /** @brief Whether this connection wire is muted. */
  muted: boolean;
  /** @brief Whether to invert the phase of audio passing through this wire (-1.0x). */
  phase_invert?: boolean;
  /** @brief Whether this wire is configured as a monitor tap. */
  is_monitor?: boolean;
}

/**
 * @brief Configuration descriptor for sidechain auto-ducking between voice and media channels.
 */
export interface AutoDuckerConfig {
  /** @brief Node ID of the voice trigger (e.g. Host Microphone). */
  trigger_node_id: string;
  /** @brief Array of target node IDs to attenuate (e.g. Spotify, Game). */
  target_node_ids: string[];
  /** @brief Voice detection threshold in dBFS (e.g. -35 dBFS). */
  threshold_db: number;
  /** @brief Attenuation applied to targets in dB (e.g. -15 dB). */
  duck_attenuation_db: number;
  /** @brief Attack ramp time in milliseconds. */
  attack_ms: number;
  /** @brief Release ramp time in milliseconds. */
  release_ms: number;
  /** @brief Whether auto-ducking is enabled. */
  enabled: boolean;
}

/**
 * @brief Data transfer object encapsulating the entire routing patchbay graph.
 */
export interface AudioGraphDto {
  /** @brief Collection of mixer nodes. */
  nodes: NodeDto[];
  /** @brief Collection of connection wires linking pins. */
  connections: ConnectionDto[];
}

/**
 * @brief Peak decibel telemetry reading for a single node.
 */
export interface NodeMeterDto {
  /** @brief Target node identifier. */
  node_id: string;
  /** @brief Left channel peak decibel amplitude (-60.0 to 0.0 dBFS). */
  peak_left: number;
  /** @brief Right channel peak decibel amplitude (-60.0 to 0.0 dBFS). */
  peak_right: number;
}

/**
 * @brief 60 FPS broadcast payload containing peak meter telemetry for all active nodes.
 */
export interface MeterUpdateDto {
  /** @brief Capture epoch timestamp in milliseconds. */
  timestamp_ms: number;
  /** @brief Array of peak meter readings for active nodes. */
  meters: NodeMeterDto[];
}

/**
 * @brief In-memory map indexing real-time peak decibel readings by node identifier.
 */
export type MeterLevels = Record<string, { left: number; right: number }>;
