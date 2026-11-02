/**
 * @file GraphCanvas.tsx
 * @brief Interactive audio routing patchbay canvas powered by React Flow.
 * @details Hosts custom Source, Output, and Virtual Device nodes with dot grid background,
 * mini-map, glowing stereo patch cords, smart auto-pairing, and 60 FPS real-time peak telemetry.
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
  } = useAudioGraph();

  // Stream 60 FPS peak decibel telemetry updates
  useMeterStream(setNodes);

  // Expose imperative methods to add nodes from header modals
  useImperativeHandle(
    ref,
    () => ({
      addSourceNode,
      addVirtualNode,
      addOutputNode,
    }),
    [addSourceNode, addVirtualNode, addOutputNode]
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
    </div>
  );
}

/**
 * @brief React Flow patchbay graph canvas component.
 */
export const GraphCanvas = forwardRef<GraphCanvasHandle, GraphCanvasProps>(GraphCanvasRender);

export default GraphCanvas;
