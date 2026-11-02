/**
 * @file initialGraph.ts
 * @brief Default initial node graph topology and patch connections for Dyno Virtual Audio Mixer.
 * @details Pre-configures sample application sources (Spotify), physical microphones (Shure MV7+),
 * virtual loopback mixing buses, and hardware playback destinations (Headphones, Studio Monitors).
 */

import type { Node, Edge } from "@xyflow/react";
import type { AudioNodeData } from "../../types/audio";

/**
 * @brief Default initial sample nodes for immediate visual patchbay demonstration.
 */
export const INITIAL_NODES: Node<AudioNodeData>[] = [
  {
    id: "src-spotify",
    type: "source",
    position: { x: 50, y: 80 },
    data: {
      id: "src-spotify",
      name: "Spotify Free",
      subtext: "PID: 1420 • Music Player",
      nodeType: "source",
      iconType: "music",
      volume: 1.0,
      muted: false,
      inputChannels: 0,
      outputChannels: 2,
      leftLevelDb: -14.2,
      rightLevelDb: -15.8,
    },
  },
  {
    id: "src-mic",
    type: "source",
    position: { x: 50, y: 380 },
    data: {
      id: "src-mic",
      name: "Shure MV7+ USB",
      subtext: "Hardware Capture Mic",
      nodeType: "source",
      iconType: "mic",
      volume: 1.0,
      muted: false,
      inputChannels: 0,
      outputChannels: 2,
      leftLevelDb: -22.5,
      rightLevelDb: -22.5,
    },
  },
  {
    id: "vdev-stream",
    type: "virtual",
    position: { x: 420, y: 220 },
    data: {
      id: "vdev-stream",
      name: "Stream Mix Bus",
      subtext: "Dyno Software Loopback",
      nodeType: "virtual",
      iconType: "virtual",
      volume: 1.0,
      muted: false,
      inputChannels: 2,
      outputChannels: 2,
      leftLevelDb: -14.2,
      rightLevelDb: -15.8,
    },
  },
  {
    id: "out-headphones",
    type: "output",
    position: { x: 800, y: 80 },
    data: {
      id: "out-headphones",
      name: "Headphones (Realtek Audio)",
      subtext: "Default Playback Endpoint",
      nodeType: "output",
      iconType: "headphones",
      volume: 1.0,
      muted: false,
      inputChannels: 2,
      outputChannels: 0,
      leftLevelDb: -14.2,
      rightLevelDb: -15.8,
    },
  },
  {
    id: "out-speakers",
    type: "output",
    position: { x: 800, y: 380 },
    data: {
      id: "out-speakers",
      name: "Studio Monitors (Scarlett 2i2)",
      subtext: "Focusrite USB Audio",
      nodeType: "output",
      iconType: "speaker",
      volume: 0.8,
      muted: false,
      inputChannels: 2,
      outputChannels: 0,
      leftLevelDb: -18.0,
      rightLevelDb: -19.5,
    },
  },
];

/**
 * @brief Helper to generate a default initial patch cable.
 * @param id Unique edge identifier.
 * @param source Source node identifier.
 * @param sourceHandle Source pin handle.
 * @param target Destination node identifier.
 * @param targetHandle Destination pin handle.
 * @param stroke Hex color code for wire glow.
 * @return Initial Edge definition.
 */
function makeSampleEdge(
  id: string,
  source: string,
  sourceHandle: string,
  target: string,
  targetHandle: string,
  stroke: string
): Edge {
  return {
    id,
    source,
    sourceHandle,
    target,
    targetHandle,
    type: "glowing",
    animated: true,
    style: { stroke, strokeWidth: 2 },
  };
}

/**
 * @brief Default initial patch cable connections between sample nodes.
 */
export const INITIAL_EDGES: Edge[] = [
  makeSampleEdge("e-spotify-headphones-L", "src-spotify", "out-0", "out-headphones", "in-0", "#38bdf8"),
  makeSampleEdge("e-spotify-headphones-R", "src-spotify", "out-1", "out-headphones", "in-1", "#60a5fa"),
  makeSampleEdge("e-spotify-stream-L", "src-spotify", "out-0", "vdev-stream", "in-0", "#38bdf8"),
  makeSampleEdge("e-spotify-stream-R", "src-spotify", "out-1", "vdev-stream", "in-1", "#60a5fa"),
  makeSampleEdge("e-mic-stream-L", "src-mic", "out-0", "vdev-stream", "in-0", "#34d399"),
  makeSampleEdge("e-mic-stream-R", "src-mic", "out-1", "vdev-stream", "in-1", "#10b981"),
  makeSampleEdge("e-stream-speakers-L", "vdev-stream", "out-0", "out-speakers", "in-0", "#a855f7"),
  makeSampleEdge("e-stream-speakers-R", "vdev-stream", "out-1", "out-speakers", "in-1", "#c084fc"),
];

/**
 * @brief Converts a React Flow channel handle string to an integer index.
 * @param handleId Handle identifier string (e.g. "out-0" or "in-1").
 * @return Channel index (0 for Left, 1 for Right).
 */
export function parseChannelIndex(handleId: string | null | undefined): number {
  if (!handleId) {
    return 0;
  }
  if (handleId.endsWith("1")) {
    return 1;
  }
  return 0;
}
