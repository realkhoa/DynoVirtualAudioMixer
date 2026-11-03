/**
 * @file initialGraph.test.ts
 * @brief Unit tests for initial patchbay topology and hardware graph construction.
 */

import { describe, expect, test } from "bun:test";
import {
  INITIAL_NODES,
  INITIAL_EDGES,
  createDefaultHardwareGraph,
  parseChannelIndex,
} from "./initialGraph";
import type { AudioDeviceDescriptor } from "../../types/audio";

describe("initialGraph", () => {
  test("defaults to a clean empty canvas without synthetic placeholders", () => {
    expect(INITIAL_NODES.length).toBe(0);
    expect(INITIAL_EDGES.length).toBe(0);
  });

  test("dynamically builds hardware graph from enumerated system devices", () => {
    const mockDevices: AudioDeviceDescriptor[] = [
      {
        id: "in-mic-1",
        name: "Microphone (USB Condenser Microphone)",
        device_type: "Input",
        channels: 2,
        sample_rate: 48000,
        is_default: true,
      },
      {
        id: "out-spk-1",
        name: "Speakers (USB Condenser Microphone)",
        device_type: "Output",
        channels: 2,
        sample_rate: 48000,
        is_default: true,
      },
    ];

    const { nodes, edges } = createDefaultHardwareGraph(mockDevices);

    expect(nodes.length).toBe(2);
    expect(nodes[0].data.nodeType).toBe("source");
    expect(nodes[0].data.name).toBe("Microphone (USB Condenser Microphone)");
    expect(nodes[1].data.nodeType).toBe("output");
    expect(nodes[1].data.name).toBe("Speakers (USB Condenser Microphone)");

    // Verifies stereo L/R auto-wiring
    expect(edges.length).toBe(2);
    expect(edges[0].sourceHandle).toBe("out-0");
    expect(edges[0].targetHandle).toBe("in-0");
    expect(edges[1].sourceHandle).toBe("out-1");
    expect(edges[1].targetHandle).toBe("in-1");
  });

  test("correctly parses channel index from handle identifiers", () => {
    expect(parseChannelIndex("out-0")).toBe(0);
    expect(parseChannelIndex("in-0")).toBe(0);
    expect(parseChannelIndex("out-1")).toBe(1);
    expect(parseChannelIndex("in-1")).toBe(1);
    expect(parseChannelIndex(null)).toBe(0);
    expect(parseChannelIndex(undefined)).toBe(0);
    expect(parseChannelIndex("")).toBe(0);
  });
});
