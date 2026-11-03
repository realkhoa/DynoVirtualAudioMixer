/**
 * @file nodeFactory.ts
 * @brief Factory helper functions to instantiate React Flow nodes with appropriate default parameters.
 */

import type { Node } from "@xyflow/react";
import type { AudioNodeData, AudioIconType } from "../../types/audio";

/**
 * @brief Node handler callbacks passed to audio nodes.
 */
export interface NodeHandlers {
  /** @brief Volume update callback. */
  onVolumeChange: (id: string, volume: number) => void;
  /** @brief Mute toggle callback. */
  onMuteToggle: (id: string, muted: boolean) => void;
  /** @brief Node removal callback. */
  onRemove: (id: string) => void;
}

const DEFAULT_HANDLERS: NodeHandlers = {
  onVolumeChange: () => {},
  onMuteToggle: () => {},
  onRemove: () => {},
};

/**
 * @brief Builds standard audio baseline metrics for new nodes.
 * @param handlers Optional node event callback handlers.
 * @return Default initial telemetry and callbacks.
 */
function createBaseMetrics(handlers?: Partial<NodeHandlers>) {
  return {
    volume: 1.0,
    muted: false,
    leftLevelDb: -60,
    rightLevelDb: -60,
    ...DEFAULT_HANDLERS,
    ...handlers,
  };
}

/**
 * @brief Constructs a new Audio Source Node.
 * @param payload Source configuration parameters.
 * @param handlers Optional node event callback handlers.
 * @return React Flow Node populated with AudioNodeData.
 */
export function createSourceNode(
  payload: { name: string; subtext: string; iconType: AudioIconType; position?: { x: number; y: number } },
  handlers?: Partial<NodeHandlers>
): Node<AudioNodeData> {
  const newId = `src-${Date.now()}`;
  return {
    id: newId,
    type: "source",
    position: payload.position || { x: 100 + Math.random() * 80, y: 150 + Math.random() * 120 },
    data: {
      id: newId,
      name: payload.name,
      subtext: payload.subtext,
      nodeType: "source",
      iconType: payload.iconType,
      inputChannels: 0,
      outputChannels: 2,
      ...createBaseMetrics(handlers),
    },
  };
}

/**
 * @brief Constructs a new Virtual Device Audio Bus Node.
 * @param payload Virtual bus configuration parameters.
 * @param handlers Optional node event callback handlers.
 * @return React Flow Node populated with AudioNodeData.
 */
export function createVirtualNode(
  payload: { name: string; subtext: string; driverType?: string; position?: { x: number; y: number } },
  handlers?: Partial<NodeHandlers>
): Node<AudioNodeData> {
  const newId = `vdev-${Date.now()}`;
  return {
    id: newId,
    type: "virtual",
    position: payload.position || { x: 440 + Math.random() * 60, y: 180 + Math.random() * 100 },
    data: {
      id: newId,
      name: payload.name,
      subtext: payload.subtext,
      nodeType: "virtual",
      iconType: "virtual",
      inputChannels: 2,
      outputChannels: 2,
      ...createBaseMetrics(handlers),
    },
  };
}

/**
 * @brief Constructs a new Output Endpoint Node.
 * @param payload Output destination configuration parameters.
 * @param handlers Optional node event callback handlers.
 * @return React Flow Node populated with AudioNodeData.
 */
export function createOutputNode(
  payload: { name: string; subtext: string; iconType: AudioIconType; position?: { x: number; y: number } },
  handlers?: Partial<NodeHandlers>
): Node<AudioNodeData> {
  const newId = `out-${Date.now()}`;
  return {
    id: newId,
    type: "output",
    position: payload.position || { x: 820 + Math.random() * 60, y: 150 + Math.random() * 120 },
    data: {
      id: newId,
      name: payload.name,
      subtext: payload.subtext,
      nodeType: "output",
      iconType: payload.iconType,
      inputChannels: 2,
      outputChannels: 0,
      ...createBaseMetrics(handlers),
    },
  };
}
