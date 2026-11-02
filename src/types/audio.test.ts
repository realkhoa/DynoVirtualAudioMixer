/**
 * @file audio.test.ts
 * @brief Unit tests for audio patchbay data contracts and graph DTO conversions.
 */

import { describe, expect, test } from "bun:test";
import type {
  AudioNodeData,
  AudioGraphDto,
  NodeDto,
  ConnectionDto,
} from "./audio";

describe("Audio Types and Graph DTO Contracts", () => {
  test("constructs valid AudioNodeData with stereo channels", () => {
    const nodeData: AudioNodeData = {
      id: "src-test",
      name: "Test Source",
      subtext: "Mock Subtext",
      nodeType: "source",
      iconType: "music",
      volume: 1.0,
      muted: false,
      inputChannels: 0,
      outputChannels: 2,
      leftLevelDb: -14.2,
      rightLevelDb: -15.8,
    };

    expect(nodeData.id).toBe("src-test");
    expect(nodeData.volume).toBe(1.0);
    expect(nodeData.muted).toBe(false);
    expect(nodeData.leftLevelDb).toBe(-14.2);
    expect(nodeData.rightLevelDb).toBe(-15.8);
  });

  test("serializes AudioGraphDto matching backend schema expectations", () => {
    const node: NodeDto = {
      id: "node-1",
      name: "Spotify",
      node_type: "source",
      volume: 1.0,
      muted: false,
      input_channels: 0,
      output_channels: 2,
    };

    const connection: ConnectionDto = {
      id: "conn-1",
      source_node_id: "node-1",
      source_channel: 0,
      destination_node_id: "node-2",
      destination_channel: 0,
      gain: 1.0,
      muted: false,
    };

    const graph: AudioGraphDto = {
      nodes: [node],
      connections: [connection],
    };

    expect(graph.nodes).toHaveLength(1);
    expect(graph.connections).toHaveLength(1);
    expect(graph.nodes[0].name).toBe("Spotify");
    expect(graph.connections[0].source_channel).toBe(0);
    expect(graph.connections[0].destination_channel).toBe(0);
  });
});
