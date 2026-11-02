/**
 * @file nodeFactory.test.ts
 * @brief Unit tests for node creation factory functions.
 */

import { describe, expect, test } from "bun:test";
import {
  createSourceNode,
  createVirtualNode,
  createOutputNode,
} from "./nodeFactory";

describe("nodeFactory", () => {
  const dummyHandlers = {
    onVolumeChange: () => {},
    onMuteToggle: () => {},
    onRemove: () => {},
  };

  test("creates valid source node with proper channel configuration", () => {
    const node = createSourceNode(
      { name: "Spotify", subtext: "PID: 1234", iconType: "music" },
      dummyHandlers
    );

    expect(node.type).toBe("source");
    expect(node.data.name).toBe("Spotify");
    expect(node.data.nodeType).toBe("source");
    expect(node.data.inputChannels).toBe(0);
    expect(node.data.outputChannels).toBe(2);
    expect(node.data.volume).toBe(1.0);
    expect(node.data.muted).toBe(false);
  });

  test("creates valid virtual node with pass-through stereo channels", () => {
    const node = createVirtualNode(
      { name: "Stream Mix", subtext: "Software Loopback" },
      dummyHandlers
    );

    expect(node.type).toBe("virtual");
    expect(node.data.name).toBe("Stream Mix");
    expect(node.data.nodeType).toBe("virtual");
    expect(node.data.inputChannels).toBe(2);
    expect(node.data.outputChannels).toBe(2);
  });

  test("creates valid output destination node", () => {
    const node = createOutputNode(
      { name: "Speakers", subtext: "USB Audio", iconType: "speaker" },
      dummyHandlers
    );

    expect(node.type).toBe("output");
    expect(node.data.name).toBe("Speakers");
    expect(node.data.nodeType).toBe("output");
    expect(node.data.inputChannels).toBe(2);
    expect(node.data.outputChannels).toBe(0);
  });
});
