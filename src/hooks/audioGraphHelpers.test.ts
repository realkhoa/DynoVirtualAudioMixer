/**
 * @file audioGraphHelpers.test.ts
 * @brief Unit tests for pure patchbay routing helpers, stereo auto-pairing, and DTO serialization.
 */

import { describe, expect, test } from "bun:test";
import type { Connection, Edge, Node } from "@xyflow/react";
import type { AudioNodeData } from "../types/audio";
import {
  isLeftChannelHandle,
  isRightChannelHandle,
  createPatchEdge,
  computeStereoPairing,
  removeEdgeFromList,
  buildAudioGraphDto,
} from "./audioGraphHelpers";

describe("audioGraphHelpers", () => {
  const stereoSourceNode: Node<AudioNodeData> = {
    id: "src-1",
    type: "source",
    position: { x: 0, y: 0 },
    data: {
      id: "src-1",
      name: "Source 1",
      nodeType: "source",
      volume: 1.0,
      muted: false,
      inputChannels: 0,
      outputChannels: 2,
      leftLevelDb: -6,
      rightLevelDb: -6,
    },
  };

  const stereoSinkNode: Node<AudioNodeData> = {
    id: "out-1",
    type: "output",
    position: { x: 300, y: 0 },
    data: {
      id: "out-1",
      name: "Sink 1",
      nodeType: "output",
      volume: 1.0,
      muted: false,
      inputChannels: 2,
      outputChannels: 0,
      leftLevelDb: -6,
      rightLevelDb: -6,
    },
  };

  const nodes = [stereoSourceNode, stereoSinkNode];

  test("isLeftChannelHandle detects channel 0 handles correctly", () => {
    expect(isLeftChannelHandle("out-0")).toBe(true);
    expect(isLeftChannelHandle("in-0")).toBe(true);
    expect(isLeftChannelHandle("out-1")).toBe(false);
    expect(isLeftChannelHandle(null)).toBe(false);
    expect(isLeftChannelHandle(undefined)).toBe(false);
  });

  test("isRightChannelHandle detects channel 1 handles correctly", () => {
    expect(isRightChannelHandle("out-1")).toBe(true);
    expect(isRightChannelHandle("in-1")).toBe(true);
    expect(isRightChannelHandle("out-0")).toBe(false);
    expect(isRightChannelHandle(null)).toBe(false);
    expect(isRightChannelHandle(undefined)).toBe(false);
  });

  test("createPatchEdge generates Left and Right channel edges with expected colors", () => {
    const leftEdge = createPatchEdge({
      source: "src-1",
      sourceHandle: "out-0",
      target: "out-1",
      targetHandle: "in-0",
    });
    expect(leftEdge.type).toBe("glowing");
    expect(leftEdge.style?.stroke).toBe("#38bdf8");

    const rightEdge = createPatchEdge(
      {
        source: "src-1",
        sourceHandle: "out-1",
        target: "out-1",
        targetHandle: "in-1",
      },
      true
    );
    expect(rightEdge.id.startsWith("twin-")).toBe(true);
    expect(rightEdge.style?.stroke).toBe("#818cf8");
  });

  test("computeStereoPairing pairs Left connections for stereo endpoints", () => {
    const connection: Connection = {
      source: "src-1",
      sourceHandle: "out-0",
      target: "out-1",
      targetHandle: "in-0",
    };
    const twin = computeStereoPairing(connection, nodes, []);
    expect(twin).not.toBeNull();
    expect(twin?.sourceHandle).toBe("out-1");
    expect(twin?.targetHandle).toBe("in-1");
  });

  test("removeEdgeFromList deletes targeted edge", () => {
    const edges: Edge[] = [
      { id: "e1", source: "a", target: "b" },
      { id: "e2", source: "c", target: "d" },
    ];
    const filtered = removeEdgeFromList(edges, "e1");
    expect(filtered.length).toBe(1);
    expect(filtered[0].id).toBe("e2");
  });

  test("buildAudioGraphDto maps nodes and edges to DTO format", () => {
    const edges: Edge[] = [
      {
        id: "e1",
        source: "src-1",
        sourceHandle: "out-0",
        target: "out-1",
        targetHandle: "in-0",
      },
    ];
    const dto = buildAudioGraphDto(nodes, edges);
    expect(dto.nodes.length).toBe(2);
    expect(dto.connections.length).toBe(1);
    expect(dto.connections[0].source_channel).toBe(0);
    expect(dto.connections[0].destination_channel).toBe(0);
  });
});
