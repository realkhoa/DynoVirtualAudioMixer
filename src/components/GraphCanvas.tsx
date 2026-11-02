/**
 * @file GraphCanvas.tsx
 * @brief Interactive audio routing patchbay canvas powered by React Flow.
 * @details Hosts custom Source, Output, and Virtual Device nodes with dot grid background,
 * mini-map, glowing stereo patch cords, and 60 FPS real-time peak decibel telemetry streaming.
 */

import {
  useCallback,
  useEffect,
  useImperativeHandle,
  useRef,
  forwardRef,
} from "react";
import {
  ReactFlow,
  Background,
  Controls,
  MiniMap,
  useNodesState,
  useEdgesState,
  addEdge,
  BackgroundVariant,
  type Connection,
  type Edge,
  type Node,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import { invoke } from "@tauri-apps/api/core";

import type {
  AudioNodeData,
  AudioIconType,
  AudioGraphDto,
  ConnectionDto,
  NodeDto,
} from "../types/audio";
import { SourceNode } from "./nodes/SourceNode";
import { OutputNode } from "./nodes/OutputNode";
import { VirtualDeviceNode } from "./nodes/VirtualDeviceNode";
import {
  INITIAL_NODES,
  INITIAL_EDGES,
  parseChannelIndex,
} from "./canvas/initialGraph";
import {
  createSourceNode,
  createVirtualNode,
  createOutputNode,
} from "./canvas/nodeFactory";
import { useMeterStream } from "./canvas/useMeterStream";

/**
 * @brief Registered React Flow custom node component catalog.
 */
const nodeTypes = {
  source: SourceNode,
  output: OutputNode,
  virtual: VirtualDeviceNode,
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
  const [nodes, setNodes, onNodesChange] = useNodesState<Node<AudioNodeData>>(INITIAL_NODES);
  const [edges, setEdges, onEdgesChange] = useEdgesState(INITIAL_EDGES);

  const nodesRef = useRef(nodes);
  nodesRef.current = nodes;

  const edgesRef = useRef(edges);
  edgesRef.current = edges;

  /**
   * @brief Dispatches updated graph state to Tauri backend routing matrix.
   */
  const syncGraphToBackend = useCallback((currentNodes: Node<AudioNodeData>[], currentEdges: Edge[]) => {
    const nodeDtos: NodeDto[] = currentNodes.map((n) => ({
      id: n.id,
      name: n.data.name,
      node_type: n.data.nodeType,
      volume: n.data.volume,
      muted: n.data.muted,
      input_channels: n.data.inputChannels,
      output_channels: n.data.outputChannels,
    }));

    const connectionDtos: ConnectionDto[] = currentEdges.map((e) => ({
      id: e.id,
      source_node_id: e.source,
      source_channel: parseChannelIndex(e.sourceHandle),
      destination_node_id: e.target,
      destination_channel: parseChannelIndex(e.targetHandle),
      gain: 1.0,
      muted: false,
    }));

    const payload: AudioGraphDto = {
      nodes: nodeDtos,
      connections: connectionDtos,
    };

    invoke("update_routing_graph", { graph: payload }).catch(() => {
      // Silently ignore in browser mock mode
    });
  }, []);

  /**
   * @brief Updates node parameter and communicates to backend.
   */
  const updateNodeParam = useCallback(
    (id: string, patch: Partial<AudioNodeData>) => {
      setNodes((nds) =>
        nds.map((node) => {
          if (node.id === id) {
            return { ...node, data: { ...node.data, ...patch } };
          }
          return node;
        })
      );

      const target = nodesRef.current.find((n) => n.id === id);
      const volume = patch.volume ?? target?.data.volume ?? 1.0;
      const muted = patch.muted ?? target?.data.muted ?? false;

      invoke("set_node_volume", { nodeId: id, volume, muted }).catch(() => {
        // Ignore in browser mode
      });
    },
    [setNodes]
  );

  const handleVolumeChange = useCallback(
    (id: string, volume: number) => updateNodeParam(id, { volume }),
    [updateNodeParam]
  );

  const handleMuteToggle = useCallback(
    (id: string, muted: boolean) => updateNodeParam(id, { muted }),
    [updateNodeParam]
  );

  /**
   * @brief Handles node deletion.
   */
  const handleRemoveNode = useCallback(
    (id: string) => {
      setNodes((nds) => {
        const nextNodes = nds.filter((n) => n.id !== id);
        setEdges((eds) => {
          const nextEdges = eds.filter((e) => e.source !== id && e.target !== id);
          syncGraphToBackend(nextNodes, nextEdges);
          return nextEdges;
        });
        return nextNodes;
      });
    },
    [setNodes, setEdges, syncGraphToBackend]
  );

  const nodeHandlers = {
    onVolumeChange: handleVolumeChange,
    onMuteToggle: handleMuteToggle,
    onRemove: handleRemoveNode,
  };

  // Enrich initial nodes with callbacks on mount
  useEffect(() => {
    setNodes((nds) =>
      nds.map((n) => ({
        ...n,
        data: { ...n.data, ...nodeHandlers },
      }))
    );
    // Run once on mount
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [handleVolumeChange, handleMuteToggle, handleRemoveNode]);

  /**
   * @brief Connects two pin handles when dragged by the user.
   */
  const onConnect = useCallback(
    (params: Connection) => {
      const isLeft = params.sourceHandle?.endsWith("0");
      const wireColor = isLeft ? "#38bdf8" : "#818cf8";

      setEdges((eds) => {
        const nextEdges = addEdge(
          {
            ...params,
            animated: true,
            style: { stroke: wireColor, strokeWidth: 2 },
          },
          eds
        );
        syncGraphToBackend(nodesRef.current, nextEdges);
        return nextEdges;
      });
    },
    [setEdges, syncGraphToBackend]
  );

  // Stream 60 FPS peak decibel updates
  useMeterStream(setNodes);

  // Expose imperative methods to add nodes
  useImperativeHandle(
    ref,
    () => ({
      addSourceNode: (payload) => {
        const node = createSourceNode(payload, nodeHandlers);
        setNodes((nds) => {
          const next = [...nds, node];
          syncGraphToBackend(next, edgesRef.current);
          return next;
        });
      },
      addVirtualNode: (payload) => {
        const node = createVirtualNode(payload, nodeHandlers);
        setNodes((nds) => {
          const next = [...nds, node];
          syncGraphToBackend(next, edgesRef.current);
          return next;
        });
      },
      addOutputNode: (payload) => {
        const node = createOutputNode(payload, nodeHandlers);
        setNodes((nds) => {
          const next = [...nds, node];
          syncGraphToBackend(next, edgesRef.current);
          return next;
        });
      },
    }),
    [handleVolumeChange, handleMuteToggle, handleRemoveNode, setNodes, syncGraphToBackend]
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
          className="!bg-slate-900/90 !border !border-slate-800 !rounded-xl !shadow-2xl overflow-hidden [&>button]:!bg-transparent [&>button]:!border-b [&>button]:!border-slate-800 [&>button]:!text-slate-300 hover:[&>button]:!bg-slate-800 [&>button]:!fill-slate-300"
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
