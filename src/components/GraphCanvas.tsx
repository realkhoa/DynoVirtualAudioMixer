/**
 * @file GraphCanvas.tsx
 * @brief Interactive audio routing patchbay canvas powered by React Flow.
 * @details Hosts custom Source, Output, and Virtual Device nodes with dot grid background,
 * mini-map, glowing stereo patch cords, smart auto-pairing, empty-state guidance, and 60 FPS real-time peak telemetry.
 */

import {
  useImperativeHandle,
  forwardRef,
} from "react";
import {
  ReactFlow,
  Background,
  Controls,
  MiniMap,
  BackgroundVariant,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import { Radio, Plus, Zap, Volume2 } from "lucide-react";

import type { AudioIconType } from "../types/audio";
import { SourceNode } from "./nodes/SourceNode";
import { OutputNode } from "./nodes/OutputNode";
import { VirtualDeviceNode } from "./nodes/VirtualDeviceNode";
import { GlowingEdge } from "./edges/GlowingEdge";
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
}

/**
 * @brief Imperative API exposed by the GraphCanvas to parent components.
 */
export interface GraphCanvasHandle {
  /** @brief Injects a new audio source node. */
  addSourceNode: (payload: { name: string; subtext: string; iconType: AudioIconType }) => void;
  /** @brief Injects a new virtual audio device node. */
  addVirtualNode: (payload: { name: string; subtext: string; driverType?: string }) => void;
  /** @brief Injects a new output destination node. */
  addOutputNode: (payload: { name: string; subtext: string; iconType: AudioIconType }) => void;
  /** @brief Clears all nodes and connections from the patchbay. */
  clearGraph: () => void;
  /** @brief Resets canvas to real detected system hardware endpoints. */
  resetToHardwareDefaults: () => Promise<void>;
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
    onNodesChange,
    onEdgesChange,
    onConnect,
    addSourceNode,
    addVirtualNode,
    addOutputNode,
    clearGraph,
    resetToHardwareDefaults,
  } = useAudioGraph();

  // Stream 60 FPS peak decibel telemetry updates
  useMeterStream(setNodes);

  // Expose imperative methods to add nodes and manipulate graph from header
  useImperativeHandle(
    ref,
    () => ({
      addSourceNode,
      addVirtualNode,
      addOutputNode,
      clearGraph,
      resetToHardwareDefaults,
    }),
    [addSourceNode, addVirtualNode, addOutputNode, clearGraph, resetToHardwareDefaults]
  );

  return (
    <div className={`relative w-full h-full bg-slate-950 overflow-hidden ${props.className || ""}`}>
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onConnect={onConnect}
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

      {/* Empty State Banner when no nodes are placed */}
      {nodes.length === 0 && (
        <div className="absolute inset-0 pointer-events-none flex items-center justify-center p-6">
          <div className="pointer-events-auto max-w-md w-full rounded-2xl bg-slate-900/90 border border-slate-800 shadow-2xl p-6 text-center backdrop-blur-md flex flex-col items-center gap-4">
            <div className="w-12 h-12 rounded-xl bg-sky-500/10 border border-sky-500/20 text-sky-400 flex items-center justify-center">
              <Radio className="w-6 h-6 animate-pulse" />
            </div>
            <div>
              <h3 className="text-base font-semibold text-white">Canvas Trống</h3>
              <p className="text-xs text-slate-400 mt-1 leading-relaxed">
                Chưa có luồng âm thanh nào được thêm vào ma trận. Bạn có thể tự động nạp thiết bị mặc định của máy hoặc thêm ứng dụng thủ công.
              </p>
            </div>
            <div className="flex flex-col w-full gap-2 pt-2">
              <button
                type="button"
                onClick={() => resetToHardwareDefaults()}
                className="flex items-center justify-center gap-2 px-4 py-2.5 rounded-xl bg-sky-500 hover:bg-sky-400 text-slate-950 text-xs font-semibold shadow-lg shadow-sky-500/20 transition-all cursor-pointer"
              >
                <Zap className="w-4 h-4 fill-current" />
                Tự Động Nạp Micro &amp; Loa Thật Của Máy
              </button>
              <div className="flex gap-2">
                <button
                  type="button"
                  onClick={() => props.onOpenAddSource?.()}
                  className="flex-1 flex items-center justify-center gap-1.5 px-3 py-2 rounded-xl bg-slate-800 hover:bg-slate-700 border border-slate-700/60 text-slate-200 text-xs font-medium transition-all cursor-pointer"
                >
                  <Plus className="w-3.5 h-3.5 text-sky-400" />
                  Thêm Nguồn
                </button>
                <button
                  type="button"
                  onClick={() => props.onOpenAddOutput?.()}
                  className="flex-1 flex items-center justify-center gap-1.5 px-3 py-2 rounded-xl bg-slate-800 hover:bg-slate-700 border border-slate-700/60 text-slate-200 text-xs font-medium transition-all cursor-pointer"
                >
                  <Volume2 className="w-3.5 h-3.5 text-emerald-400" />
                  Thêm Đầu Ra
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

/**
 * @brief React Flow patchbay graph canvas component.
 */
export const GraphCanvas = forwardRef<GraphCanvasHandle, GraphCanvasProps>(GraphCanvasRender);

export default GraphCanvas;
