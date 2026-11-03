/**
 * @file initialGraph.ts
 * @brief Default initial node graph topology and hardware discovery helpers for Dyno Virtual Audio Mixer.
 * @details Defaults to a clean empty patchbay canvas without synthetic placeholder devices,
 * providing factory utilities to initialize real hardware devices detected on the user's system.
 */

import type { Node, Edge } from "@xyflow/react";
import type { AudioNodeData, AudioDeviceDescriptor } from "../../types/audio";
import { createSourceNode, createOutputNode } from "./nodeFactory";
import { createPatchEdge } from "../../hooks/audioGraphHelpers";

/**
 * @brief Default initial nodes collection: empty canvas by default to avoid synthetic mockups.
 */
export const INITIAL_NODES: Node<AudioNodeData>[] = [];

/**
 * @brief Default initial patch cables: empty by default.
 */
export const INITIAL_EDGES: Edge[] = [];

/**
 * @brief Constructs a clean patchbay topology dynamically populated from real system audio endpoints.
 * @details Automatically detects the system default microphone input and default speaker/headphone
 * output, positioning them naturally on the canvas and wiring Left and Right stereo channels.
 * @param devices List of real audio devices enumerated from the host system.
 * @return {{ nodes: Node<AudioNodeData>[]; edges: Edge[] }} Real hardware initial graph.
 */
export function createDefaultHardwareGraph(devices: AudioDeviceDescriptor[]): {
  nodes: Node<AudioNodeData>[];
  edges: Edge[];
} {
  const nodes: Node<AudioNodeData>[] = [];
  const edges: Edge[] = [];

  const defaultInput =
    devices.find((d) => d.device_type === "Input" && d.is_default) ||
    devices.find((d) => d.device_type === "Input");

  const defaultOutput =
    devices.find((d) => d.device_type === "Output" && d.is_default) ||
    devices.find((d) => d.device_type === "Output");

  if (defaultInput) {
    const inputNode = createSourceNode({
      name: defaultInput.name,
      subtext: `Physical Mic • ${defaultInput.sample_rate} Hz • ${defaultInput.channels} Ch`,
      iconType: "mic",
      position: { x: 80, y: 160 },
    });
    nodes.push(inputNode);
  }

  if (defaultOutput) {
    const isPhones = defaultOutput.name.toLowerCase().includes("headphone");
    const outputNode = createOutputNode({
      name: defaultOutput.name,
      subtext: `Default Output • ${defaultOutput.sample_rate} Hz • ${defaultOutput.channels} Ch`,
      iconType: isPhones ? "headphones" : "speaker",
      position: { x: 750, y: 160 },
    });
    nodes.push(outputNode);
  }

  // Auto-connect stereo channels if both default input and output exist
  if (nodes.length === 2) {
    const src = nodes[0].id;
    const dst = nodes[1].id;
    edges.push(
      createPatchEdge({
        source: src,
        sourceHandle: "out-0",
        target: dst,
        targetHandle: "in-0",
      })
    );
    edges.push(
      createPatchEdge(
        {
          source: src,
          sourceHandle: "out-1",
          target: dst,
          targetHandle: "in-1",
        },
        true
      )
    );
  }

  return { nodes, edges };
}

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
