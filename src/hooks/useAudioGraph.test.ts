/**
 * @file useAudioGraph.test.ts
 * @brief Unit tests for patchbay graph management, smart stereo auto-pairing, and IPC serialization.
 */

import { describe, expect, test } from "bun:test";
import type { Node, Edge, Connection } from "@xyflow/react";
import type { AudioNodeData } from "../types/audio";
import {
  useAudioGraph,
  computeStereoPairing,
  createPatchEdge,
  removeEdgeFromList,
  buildAudioGraphDto,
  isLeftChannelHandle,
  isRightChannelHandle,
} from "./useAudioGraph";

describe("useAudioGraph and Smart Stereo Auto-Pairing", () => {
  const stereoSourceNode: Node<AudioNodeData> = {
    id: "src-music",
    type: "source",
    position: { x: 0, y: 0 },
    data: {
      id: "src-music",
      name: "Music Player",
      nodeType: "source",
      volume: 1.0,
      muted: false,
      inputChannels: 0,
      outputChannels: 2,
      leftLevelDb: -10,
      rightLevelDb: -10,
    },
  };

  const stereoSinkNode: Node<AudioNodeData> = {
    id: "out-speakers",
    type: "output",
    position: { x: 400, y: 0 },
    data: {
      id: "out-speakers",
      name: "Stereo Speakers",
      nodeType: "output",
      volume: 1.0,
      muted: false,
      inputChannels: 2,
      outputChannels: 0,
      leftLevelDb: -10,
      rightLevelDb: -10,
    },
  };

  const monoSourceNode: Node<AudioNodeData> = {
    id: "src-mono-mic",
    type: "source",
    position: { x: 0, y: 200 },
    data: {
      id: "src-mono-mic",
      name: "Mono Microphone",
      nodeType: "source",
      volume: 1.0,
      muted: false,
      inputChannels: 0,
      outputChannels: 1,
      leftLevelDb: -20,
      rightLevelDb: -60,
    },
  };

  const monoSinkNode: Node<AudioNodeData> = {
    id: "out-mono-phone",
    type: "output",
    position: { x: 400, y: 200 },
    data: {
      id: "out-mono-phone",
      name: "Mono Phone Sink",
      nodeType: "output",
      volume: 1.0,
      muted: false,
      inputChannels: 1,
      outputChannels: 0,
      leftLevelDb: -20,
      rightLevelDb: -60,
    },
  };

  const nodes = [stereoSourceNode, stereoSinkNode, monoSourceNode, monoSinkNode];

  const standardStereoLeftConn: Connection = {
    source: "src-music",
    sourceHandle: "out-0",
    target: "out-speakers",
    targetHandle: "in-0",
  };

  test("defines useAudioGraph hook function", () => {
    expect(typeof useAudioGraph).toBe("function");
  });

  test("correctly identifies Left and Right channel handles", () => {
    expect(isLeftChannelHandle("out-0")).toBe(true);
    expect(isLeftChannelHandle("in-0")).toBe(true);
    expect(isLeftChannelHandle("out-1")).toBe(false);
    expect(isLeftChannelHandle("in-1")).toBe(false);
    expect(isLeftChannelHandle(null)).toBe(false);
    expect(isLeftChannelHandle(undefined)).toBe(false);

    expect(isRightChannelHandle("out-1")).toBe(true);
    expect(isRightChannelHandle("in-1")).toBe(true);
    expect(isRightChannelHandle("out-0")).toBe(false);
    expect(isRightChannelHandle("in-0")).toBe(false);
    expect(isRightChannelHandle(null)).toBe(false);
    expect(isRightChannelHandle(undefined)).toBe(false);
  });

  test("smart auto-pairing: connecting Left to Left produces Right-channel twin edge", () => {
    const existingEdges: Edge[] = [];
    const twin = computeStereoPairing(standardStereoLeftConn, nodes, existingEdges);

    expect(twin).not.toBeNull();
    expect(twin?.source).toBe("src-music");
    expect(twin?.sourceHandle).toBe("out-1");
    expect(twin?.target).toBe("out-speakers");
    expect(twin?.targetHandle).toBe("in-1");
    expect(twin?.type).toBe("glowing");
  });

  test("smart auto-pairing: single channel connection out-0 to in-1 produces NO twin edge", () => {
    const crossConnection: Connection = {
      source: "src-music",
      sourceHandle: "out-0",
      target: "out-speakers",
      targetHandle: "in-1",
    };

    const existingEdges: Edge[] = [];
    const twin = computeStereoPairing(crossConnection, nodes, existingEdges);
    expect(twin).toBeNull();
  });

  test("smart auto-pairing: connection to mono destination produces NO twin edge", () => {
    const monoConnection: Connection = {
      source: "src-music",
      sourceHandle: "out-0",
      target: "out-mono-phone",
      targetHandle: "in-0",
    };

    const existingEdges: Edge[] = [];
    const twin = computeStereoPairing(monoConnection, nodes, existingEdges);
    expect(twin).toBeNull();
  });

  test("smart auto-pairing: connection from mono source produces NO twin edge", () => {
    const monoConnection: Connection = {
      source: "src-mono-mic",
      sourceHandle: "out-0",
      target: "out-speakers",
      targetHandle: "in-0",
    };

    const existingEdges: Edge[] = [];
    const twin = computeStereoPairing(monoConnection, nodes, existingEdges);
    expect(twin).toBeNull();
  });

  test("smart auto-pairing: does not duplicate twin if Right edge already exists", () => {
    const existingEdges: Edge[] = [
      {
        id: "e-existing-R",
        source: "src-music",
        sourceHandle: "out-1",
        target: "out-speakers",
        targetHandle: "in-1",
        type: "glowing",
      },
    ];

    const twin = computeStereoPairing(standardStereoLeftConn, nodes, existingEdges);
    expect(twin).toBeNull();
  });

  test("createPatchEdge generates properly styled glowing edge", () => {
    const leftEdge = createPatchEdge(standardStereoLeftConn);

    expect(leftEdge.id).toBeTruthy();
    expect(leftEdge.type).toBe("glowing");
    expect(leftEdge.source).toBe("src-music");
    expect(leftEdge.target).toBe("out-speakers");
    expect(leftEdge.animated).toBe(true);
    expect(leftEdge.style?.stroke).toBe("#38bdf8");

    const rightEdge = createPatchEdge({
      source: "src-music",
      sourceHandle: "out-1",
      target: "out-speakers",
      targetHandle: "in-1",
    });
    expect(rightEdge.style?.stroke).toBe("#818cf8");
  });

  test("removeEdgeFromList accurately removes target edge by ID", () => {
    const edges: Edge[] = [
      { id: "e1", source: "a", target: "b" },
      { id: "e2", source: "c", target: "d" },
      { id: "e3", source: "e", target: "f" },
    ];

    const updated = removeEdgeFromList(edges, "e2");
    expect(updated.length).toBe(2);
    expect(updated.find((e) => e.id === "e2")).toBeUndefined();
    expect(updated[0].id).toBe("e1");
    expect(updated[1].id).toBe("e3");
  });

  test("buildAudioGraphDto transforms nodes and edges into backend DTO format", () => {
    const testEdges: Edge[] = [
      {
        id: "wire-1",
        source: "src-music",
        sourceHandle: "out-0",
        target: "out-speakers",
        targetHandle: "in-0",
      },
      {
        id: "wire-2",
        source: "src-music",
        sourceHandle: "out-1",
        target: "out-speakers",
        targetHandle: "in-1",
      },
    ];

    const dto = buildAudioGraphDto(nodes, testEdges);

    expect(dto.nodes.length).toBe(4);
    expect(dto.connections.length).toBe(2);

    expect(dto.connections[0].id).toBe("wire-1");
    expect(dto.connections[0].source_node_id).toBe("src-music");
    expect(dto.connections[0].source_channel).toBe(0);
    expect(dto.connections[0].destination_node_id).toBe("out-speakers");
    expect(dto.connections[0].destination_channel).toBe(0);
    expect(dto.connections[0].gain).toBe(1.0);
    expect(dto.connections[0].muted).toBe(false);

    expect(dto.connections[1].id).toBe("wire-2");
    expect(dto.connections[1].source_channel).toBe(1);
    expect(dto.connections[1].destination_channel).toBe(1);
  });
});
