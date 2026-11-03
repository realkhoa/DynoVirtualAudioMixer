/**
 * @file GraphCanvas.tsx
 * @brief Interactive audio routing patchbay canvas powered by React Flow.
 * @details Hosts custom Source, Output, and Virtual Device nodes with dot grid background,
 * mini-map, glowing stereo patch cords, smart auto-pairing, empty-state guidance, 60 FPS real-time peak telemetry,
 * Wire Inspector dialog, Sidechain Auto-Ducking modal, and floating compact Mini Bar overlay.
 */

import {
  useState,
  useImperativeHandle,
  forwardRef,
  useCallback,
} from "react";
import {
  ReactFlow,
  Background,
  Controls,
  MiniMap,
  BackgroundVariant,
  type Edge,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import { invoke } from "@tauri-apps/api/core";

import type {
  AudioIconType,
  AutoDuckerConfig,
  NodeDto,
} from "../types/audio";
import { SourceNode } from "./nodes/SourceNode";
import { OutputNode } from "./nodes/OutputNode";
import { VirtualDeviceNode } from "./nodes/VirtualDeviceNode";
import { GlowingEdge } from "./edges/GlowingEdge";
import { EdgeInspectorModal } from "./edges/EdgeInspectorModal";
import { DuckingModal } from "./modals/DuckingModal";
import { MiniBar } from "./compact/MiniBar";
import { EmptyGraphBanner } from "./canvas/EmptyGraphBanner";
import { useMeterStream } from "./canvas/useMeterStream";
import { useAudioGraph } from "../hooks/useAudioGraph";

/**
 * @brief Registered React Flow custom node component catalog.
 */
const nodeTypes = {
  source: SourceNode,
  output: OutputNode,
  virtual: VirtualDeviceNode,
};

/**
 * @brief Registered React Flow custom edge component catalog.
 */
const edgeTypes = {
  glowing: GlowingEdge,
};

/**
 * @brief Props definition for the GraphCanvas component.
 */
export interface GraphCanvasProps {
  /** @brief Optional additional CSS class names. */
  className?: string;
  /** @brief Request to open Add Source modal from empty state. */
  onOpenAddSource?: () => void;
  /** @brief Request to open Add Output modal from empty state. */
  onOpenAddOutput?: () => void;
  /** @brief Callback when mini bar active state toggles. */
  onMiniBarChange?: (active: boolean) => void;
  /** @brief Callback when auto-ducking active state toggles. */
  onDuckingChange?: (active: boolean) => void;
}

/**
 * @brief Imperative API exposed by the GraphCanvas to parent components.
 */
export interface GraphCanvasHandle {
  /** @brief Injects a new audio source node. */
  addSourceNode: (payload: { name: string; subtext: string; iconType: AudioIconType }) => void;
  /** @brief Injects a new virtual audio device node. */
  addVirtualNode: (payload: { name: string; subtext: string; driverType?: string; channels?: number }) => void;
  /** @brief Injects a new output destination node. */
  addOutputNode: (payload: { name: string; subtext: string; iconType: AudioIconType }) => void;
  /** @brief Clears all nodes and connections from the patchbay. */
  clearGraph: () => void;
  /** @brief Resets canvas to real detected system hardware endpoints. */
  resetToHardwareDefaults: () => Promise<void>;
  /** @brief Opens the Sidechain Auto-Ducking configuration modal. */
  openDuckingModal: () => void;
  /** @brief Toggles the floating compact Mini Bar overlay. */
  toggleMiniBar: () => void;
}

/**
 * @brief Inner patchbay canvas render function.
 * @param props Component properties.
 * @param ref Forwarded imperative handle for injecting nodes.
 * @return {React.JSX.Element} Rendered patchbay canvas.
 */
function GraphCanvasRender(
  props: GraphCanvasProps,
  ref: React.ForwardedRef<GraphCanvasHandle>
): React.JSX.Element {
  const {
    nodes,
    edges,
    setNodes,
    setEdges,
    onNodesChange,
    onEdgesChange,
    onConnect,
    handleVolumeChange,
    handleMuteToggle,
    addSourceNode,
    addVirtualNode,
    addOutputNode,
    clearGraph,
    resetToHardwareDefaults,
    syncGraphToBackend,
  } = useAudioGraph();

  // Stream 60 FPS peak decibel telemetry updates
  useMeterStream(setNodes);

  // Inspector & Modal states
  const [inspectingEdgeId, setInspectingEdgeId] = useState<string | null>(null);
  const [isDuckingOpen, setIsDuckingOpen] = useState(false);
  const [duckingConfig, setDuckingConfig] = useState<AutoDuckerConfig | null>(null);
  const [isMiniBarActive, setIsMiniBarActive] = useState(false);

  // Expose imperative methods to add nodes and manipulate graph from header
  useImperativeHandle(
    ref,
    () => ({
      addSourceNode,
      addVirtualNode,
      addOutputNode,
      clearGraph,
      resetToHardwareDefaults,
      openDuckingModal: () => setIsDuckingOpen(true),
      toggleMiniBar: () => {
        setIsMiniBarActive((prev) => {
          const next = !prev;
          props.onMiniBarChange?.(next);
          return next;
        });
      },
    }),
    [addSourceNode, addVirtualNode, addOutputNode, clearGraph, resetToHardwareDefaults, props]
  );

  // Selected edge object for inspector
  const inspectingEdge = edges.find((e) => e.id === inspectingEdgeId);
  const sourceNode = nodes.find((n) => n.id === inspectingEdge?.source);
  const targetNode = nodes.find((n) => n.id === inspectingEdge?.target);

  /**
   * @brief Saves edited wire parameters, updates React Flow state, and syncs to backend.
   * @param gain Linear gain multiplier.
   * @param muted Mute status of the wire.
   * @param phaseInvert Whether 180° phase inversion is active.
   */
  const handleSaveWire = useCallback(
    (gain: number, muted: boolean, phaseInvert: boolean) => {
      if (!inspectingEdgeId) return;

      setEdges((eds) => {
        const next = eds.map((e) => {
          if (e.id === inspectingEdgeId) {
            return {
              ...e,
              data: {
                ...e.data,
                gain,
                muted,
                phaseInvert,
              },
            };
          }
          return e;
        });
        syncGraphToBackend(nodes, next);
        return next;
      });

      invoke("set_wire_config", {
        wireId: inspectingEdgeId,
        gain,
        muted,
        phaseInvert,
        isMonitor: false,
      }).catch(() => {
        // Fallback in browser preview
      });
    },
    [inspectingEdgeId, nodes, setEdges, syncGraphToBackend]
  );

  /**
   * @brief Saves sidechain auto-ducking configuration.
   * @param config AutoDuckerConfig payload.
   */
  const handleSaveDucking = useCallback(
    (config: AutoDuckerConfig) => {
      setDuckingConfig(config);
      props.onDuckingChange?.(config.enabled);
    },
    [props]
  );

  // Map nodes to NodeDto for ducking modal selector
  const nodeDtos: NodeDto[] = nodes.map((n) => ({
    id: n.id,
    name: n.data.name,
    node_type: n.data.nodeType,
    volume: n.data.volume,
    muted: n.data.muted,
    input_channels: n.data.inputChannels,
    output_channels: n.data.outputChannels,
    is_monitor: n.data.isMonitor,
    mute_when_monitoring: n.data.muteWhenMonitoring,
    is_offline: n.data.isOffline,
  }));

  return (
    <div className={`relative w-full h-full bg-slate-950 overflow-hidden ${props.className || ""}`}>
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onConnect={onConnect}
        onEdgeClick={(_evt, edge: Edge) => setInspectingEdgeId(edge.id)}
        nodeTypes={nodeTypes}
        edgeTypes={edgeTypes}
        defaultEdgeOptions={{ type: "glowing" }}
        fitView
        fitViewOptions={{ padding: 0.2 }}
        minZoom={0.2}
        maxZoom={2.0}
        className="bg-[#0a0f1d]"
      >
        <Background
          variant={BackgroundVariant.Dots}
          gap={18}
          size={1.5}
          color="#334155"
          className="opacity-70"
        />
        <Controls
          className={
            "!bg-slate-900/90 !border !border-slate-800 !rounded-xl !shadow-2xl overflow-hidden " +
            "[&>button]:!bg-transparent [&>button]:!border-b [&>button]:!border-slate-800 " +
            "[&>button]:!text-slate-300 hover:[&>button]:!bg-slate-800 [&>button]:!fill-slate-300"
          }
          showInteractive={false}
        />
        <MiniMap
          className="!bg-slate-900/90 !border !border-slate-800 !rounded-xl !shadow-2xl overflow-hidden"
          nodeColor={(n) => {
            if (n.type === "source") return "#38bdf8";
            if (n.type === "output") return "#10b981";
            return "#a855f7";
          }}
          maskColor="rgba(10, 15, 29, 0.75)"
        />
      </ReactFlow>

      {/* Floating Compact Mini Bar Overlay */}
      {isMiniBarActive && (
        <MiniBar
          nodes={nodes}
          onVolumeChange={handleVolumeChange}
          onMuteToggle={handleMuteToggle}
          onExpandToGraph={() => {
            setIsMiniBarActive(false);
            props.onMiniBarChange?.(false);
          }}
          onOpenDucking={() => setIsDuckingOpen(true)}
          isDuckingActive={Boolean(duckingConfig?.enabled)}
        />
      )}

      {/* Connection Wire Inspector Dialog */}
      <EdgeInspectorModal
        isOpen={Boolean(inspectingEdgeId && inspectingEdge)}
        onClose={() => setInspectingEdgeId(null)}
        edgeId={inspectingEdgeId || ""}
        sourceName={sourceNode?.data.name}
        destName={targetNode?.data.name}
        initialGain={(inspectingEdge?.data as any)?.gain ?? 1.0}
        initialMuted={Boolean((inspectingEdge?.data as any)?.muted)}
        initialPhaseInvert={Boolean((inspectingEdge?.data as any)?.phaseInvert)}
        onSave={handleSaveWire}
      />

      {/* Sidechain Auto-Ducking Configuration Dialog */}
      <DuckingModal
        isOpen={isDuckingOpen}
        onClose={() => setIsDuckingOpen(false)}
        nodes={nodeDtos}
        currentConfig={duckingConfig}
        onSave={handleSaveDucking}
      />

      {/* Empty State Banner when no nodes are placed */}
      {nodes.length === 0 && (
        <EmptyGraphBanner
          onResetHardware={() => resetToHardwareDefaults()}
          onOpenAddSource={props.onOpenAddSource}
          onOpenAddOutput={props.onOpenAddOutput}
        />
      )}
    </div>
  );
}

/**
 * @brief React Flow patchbay graph canvas component.
 */
export const GraphCanvas = forwardRef<GraphCanvasHandle, GraphCanvasProps>(GraphCanvasRender);

export default GraphCanvas;
