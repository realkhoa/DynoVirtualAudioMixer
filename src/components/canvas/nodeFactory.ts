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
 * @param handlers? Optional node event callback handlers.
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
  payload: {
    name: string;
    subtext: string;
    iconType: AudioIconType;
    nodeType?: string;
    pid?: number;
    channels?: number;
    isOffline?: boolean;
    noiseGateEnabled?: boolean;
    noiseGateThresholdDb?: number;
    position?: { x: number; y: number };
  },
  handlers?: Partial<NodeHandlers>
): Node<AudioNodeData> {
  const isMic = payload.iconType === "mic";
  const newId = isMic ? `mic-${Date.now()}` : `app-${Date.now()}`;
  return {
    id: newId,
    type: "source",
    position: payload.position || { x: 100 + Math.random() * 80, y: 150 + Math.random() * 120 },
    data: {
      id: newId,
      name: payload.name,
      subtext: payload.subtext,
      nodeType: (payload.nodeType || "source") as any,
      iconType: payload.iconType,
      pid: payload.pid,
      inputChannels: 0,
      outputChannels: payload.channels ?? 2,
      isOffline: payload.isOffline ?? false,
      noiseGateEnabled: payload.noiseGateEnabled ?? false,
      noiseGateThresholdDb: payload.noiseGateThresholdDb ?? -45.0,
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
  payload: {
    name: string;
    subtext: string;
    driverType?: string;
    channels?: number;
    position?: { x: number; y: number };
  },
  handlers?: Partial<NodeHandlers>
): Node<AudioNodeData> {
  const newId = `vdev-${Date.now()}`;
  const ch = payload.channels ?? 2;
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
      inputChannels: ch,
      outputChannels: ch,
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
  payload: {
    name: string;
    subtext: string;
    iconType: AudioIconType;
    channels?: number;
    isMonitor?: boolean;
    muteWhenMonitoring?: boolean;
    isOffline?: boolean;
    position?: { x: number; y: number };
  },
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
      inputChannels: payload.channels ?? 2,
      outputChannels: 0,
      isMonitor: payload.isMonitor ?? false,
      muteWhenMonitoring: payload.muteWhenMonitoring ?? false,
      isOffline: payload.isOffline ?? false,
      ...createBaseMetrics(handlers),
    },
  };
}
