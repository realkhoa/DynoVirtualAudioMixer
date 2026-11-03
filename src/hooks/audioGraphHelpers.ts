/**
 * @file audioGraphHelpers.ts
 * @brief Pure helper functions for patchbay graph routing, smart stereo auto-pairing, and DTO serialization.
 * @details Implements channel handle identification, twin edge calculation for stereo Left/Right pairs,
 * edge removal from collections, and conversion between React Flow topologies and backend AudioGraphDto schemas.
 */

import { invoke } from "@tauri-apps/api/core";
import type { Connection, Edge, Node } from "@xyflow/react";
import type { AudioNodeData, AudioGraphDto, ConnectionDto, NodeDto, AudioDeviceDescriptor } from "../types/audio";
import { parseChannelIndex, createDefaultHardwareGraph } from "../components/canvas/initialGraph";
import type { NodeHandlers } from "../components/canvas/nodeFactory";

/**
 * @brief Determines if a given handle identifier represents a Left audio channel (channel 0).
 * @param handleId Handle identifier string (e.g. "out-0", "in-0").
 * @return True if handle corresponds to channel 0.
 */
export function isLeftChannelHandle(handleId: string | null | undefined): boolean {
  if (!handleId) {
    return false;
  }
  return handleId.endsWith("0");
}

/**
 * @brief Determines if a given handle identifier represents a Right audio channel (channel 1).
 * @param handleId Handle identifier string (e.g. "out-1", "in-1").
 * @return True if handle corresponds to channel 1.
 */
export function isRightChannelHandle(handleId: string | null | undefined): boolean {
  if (!handleId) {
    return false;
  }
  return handleId.endsWith("1");
}

/**
 * @brief Creates a styled React Flow patch edge for audio routing.
 * @param connection User connection parameters.
 * @param isTwin Whether this is an auto-paired twin edge.
 * @return Styled Edge object with glowing custom edge type.
 */
export function createPatchEdge(connection: Connection, isTwin: boolean = false): Edge {
  const isLeft = isLeftChannelHandle(connection.sourceHandle);
  const wireColor = isLeft ? "#38bdf8" : "#818cf8";
  const prefix = isTwin ? "twin" : "wire";
  const rand = Math.random().toString(36).substring(2, 6);
  const handleTag = connection.sourceHandle || "0";
  const edgeId = `${prefix}-${connection.source}-${connection.target}-${handleTag}-${Date.now()}-${rand}`;

  return {
    id: edgeId,
    source: connection.source,
    sourceHandle: connection.sourceHandle,
    target: connection.target,
    targetHandle: connection.targetHandle,
    type: "glowing",
    animated: true,
    style: { stroke: wireColor, strokeWidth: 2 },
    data: {
      channel: isLeft ? 0 : 1,
      active: true,
    },
  };
}

/**
 * @brief Evaluates whether connecting two Left pins triggers automatic Right-channel twin pairing.
 * @details If source pin is `out-0` and target pin is `in-0`, and both source and destination
 * nodes support stereo routing (>= 2 channels), this produces a twin edge for `out-1` -> `in-1`.
 * @param connection The candidate connection created by user interaction.
 * @param nodes Current nodes collection in the graph.
 * @param existingEdges Current edges collection in the graph.
 * @return Twin Edge if auto-pairing is valid; otherwise null.
 */
export function computeStereoPairing(
  connection: Connection,
  nodes: Node<AudioNodeData>[],
  existingEdges: Edge[]
): Edge | null {
  // Only Left-to-Left connections initiate smart stereo auto-pairing
  if (
    !isLeftChannelHandle(connection.sourceHandle) ||
    !isLeftChannelHandle(connection.targetHandle)
  ) {
    return null;
  }

  const sourceNode = nodes.find((n) => n.id === connection.source);
  const targetNode = nodes.find((n) => n.id === connection.target);

  if (!sourceNode || !targetNode) {
    return null;
  }

  // Verify both endpoints support at least stereo (2 channels)
  const sourceOutputChannels = sourceNode.data?.outputChannels ?? 0;
  const targetInputChannels = targetNode.data?.inputChannels ?? 0;

  if (sourceOutputChannels < 2 || targetInputChannels < 2) {
    return null;
  }

  // Check if right channel twin is already connected to avoid duplicate cables
  const twinAlreadyExists = existingEdges.some(
    (e) =>
      e.source === connection.source &&
      e.sourceHandle === "out-1" &&
      e.target === connection.target &&
      e.targetHandle === "in-1"
  );

  if (twinAlreadyExists) {
    return null;
  }

  const twinId = `twin-${connection.source}-${connection.target}-out-1-in-1-${Date.now()}`;
  return {
    id: twinId,
    source: connection.source,
    sourceHandle: "out-1",
    target: connection.target,
    targetHandle: "in-1",
    type: "glowing",
    animated: true,
    style: { stroke: "#818cf8", strokeWidth: 2 },
    data: {
      channel: 1,
      active: true,
    },
  };
}

/**
 * @brief Removes an edge from an existing edge list by its unique identifier.
 * @param edges Original collection of patch edges.
 * @param edgeId Unique edge identifier to remove.
 * @return New array without the specified edge.
 */
export function removeEdgeFromList(edges: Edge[], edgeId: string): Edge[] {
  return edges.filter((e) => e.id !== edgeId);
}

/**
 * @brief Serializes React Flow graph topology to backend AudioGraphDto.
 * @param nodes Array of audio graph nodes.
 * @param edges Array of patch edges.
 * @return Serialized AudioGraphDto payload for Tauri IPC.
 */
export function buildAudioGraphDto(
  nodes: Node<AudioNodeData>[],
  edges: Edge[]
): AudioGraphDto {
  const nodeDtos: NodeDto[] = nodes.map((n) => ({
    id: n.id,
    name: n.data.name,
    node_type: n.data.nodeType,
    volume: n.data.volume,
    muted: n.data.muted,
    input_channels: n.data.inputChannels,
    output_channels: n.data.outputChannels,
    is_monitor: n.data.isMonitor ?? false,
    mute_when_monitoring: n.data.muteWhenMonitoring ?? false,
    is_offline: n.data.isOffline ?? false,
  }));

  const connectionDtos: ConnectionDto[] = edges.map((e) => ({
    id: e.id,
    source_node_id: e.source,
    source_channel: parseChannelIndex(e.sourceHandle),
    destination_node_id: e.target,
    destination_channel: parseChannelIndex(e.targetHandle),
    gain: typeof (e.data as any)?.gain === "number" ? (e.data as any).gain : 1.0,
    muted: Boolean((e.data as any)?.muted),
    phase_invert: Boolean((e.data as any)?.phaseInvert),
    is_monitor: Boolean((e.data as any)?.isMonitor),
  }));

  return {
    nodes: nodeDtos,
    connections: connectionDtos,
  };
}

/**
 * @brief LocalStorage cache key for persisting graph topology between app restarts.
 */
export const STORAGE_KEY = "dyno_mixer_graph_state_v2";

/**
 * @brief Attempts to load persisted graph topology from browser localStorage.
 * @return {{ nodes: Node<AudioNodeData>[]; edges: Edge[] } | null} Restored graph or null.
 */
export function loadPersistedGraph(): { nodes: Node<AudioNodeData>[]; edges: Edge[] } | null {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw);
    if (Array.isArray(parsed?.nodes) && Array.isArray(parsed?.edges)) {
      return { nodes: parsed.nodes, edges: parsed.edges };
    }
  } catch {
    // Ignore storage parse errors
  }
  return null;
}

/**
 * @brief Persists current graph topology into browser localStorage.
 * @param nodes Current nodes array.
 * @param edges Current edges array.
 */
export function savePersistedGraph(
  nodes: Node<AudioNodeData>[],
  edges: Edge[]
): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify({ nodes, edges }));
  } catch {
    // Ignore storage write errors
  }
}

/**
 * @brief Queries hardware devices from backend and constructs default hardware patch graph.
 * @param nodeHandlers Node lifecycle handlers.
 * @param onDelete Edge deletion handler.
 * @return Constructed nodes and edges or null.
 */
export async function fetchDefaultHardwareGraph(
  nodeHandlers: NodeHandlers,
  onDelete: (edgeId: string) => void
): Promise<{ nodes: Node<AudioNodeData>[]; edges: Edge[] } | null> {
  try {
    const realDevices = await invoke<AudioDeviceDescriptor[]>("get_devices");
    if (Array.isArray(realDevices) && realDevices.length > 0) {
      const { nodes: hwNodes, edges: hwEdges } = createDefaultHardwareGraph(realDevices);
      return {
        nodes: hwNodes.map((n) => ({ ...n, data: { ...n.data, ...nodeHandlers } })),
        edges: hwEdges.map((e) => ({ ...e, data: { ...e.data, onDelete } })),
      };
    }
  } catch {
    // Fallback if outside Tauri
  }
  return null;
}
