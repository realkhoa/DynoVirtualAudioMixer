/**
 * @file initialGraph.test.ts
 * @brief Unit tests for initial patchbay topology and channel index parsing.
 */

import { describe, expect, test } from "bun:test";
import {
  INITIAL_NODES,
  INITIAL_EDGES,
  parseChannelIndex,
} from "./initialGraph";

describe("initialGraph", () => {
  test("contains default sample nodes for sources, virtual buses, and outputs", () => {
    expect(INITIAL_NODES.length).toBeGreaterThanOrEqual(5);

    const spotifyNode = INITIAL_NODES.find((n) => n.id === "src-spotify");
    expect(spotifyNode).toBeDefined();
    expect(spotifyNode?.data.nodeType).toBe("source");

    const vdevNode = INITIAL_NODES.find((n) => n.id === "vdev-stream");
    expect(vdevNode).toBeDefined();
    expect(vdevNode?.data.nodeType).toBe("virtual");

    const outNode = INITIAL_NODES.find((n) => n.id === "out-headphones");
    expect(outNode).toBeDefined();
    expect(outNode?.data.nodeType).toBe("output");
  });

  test("contains initial glowing patch cables", () => {
    expect(INITIAL_EDGES.length).toBeGreaterThanOrEqual(4);
    for (const edge of INITIAL_EDGES) {
      expect(edge.source).toBeDefined();
      expect(edge.target).toBeDefined();
      expect(edge.animated).toBe(true);
    }
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
