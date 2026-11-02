/**
 * @file useAudioGraph.ts
 * @brief State management hook for the audio patchbay graph, smart stereo pairing, and Tauri synchronization.
 * @details Encapsulates React Flow node and edge state, automated Right-channel twin pairing when Left
 * channels are connected, real-time volume/mute parameter dispatching, and matrix graph topology synchronization.
 */

import { useCallback, useEffect, useRef } from "react";
import {
  useNodesState,
  useEdgesState,
  type Connection,
  type Edge,
  type Node,
  type OnNodesChange,
  type OnEdgesChange,
  type EdgeChange,
} from "@xyflow/react";
import { invoke } from "@tauri-apps/api/core";

import type { AudioNodeData, AudioIconType } from "../types/audio";
import { INITIAL_NODES, INITIAL_EDGES } from "../components/canvas/initialGraph";
import {
  createSourceNode,
  createVirtualNode,
  createOutputNode,
} from "../components/canvas/nodeFactory";
export * from "./audioGraphHelpers";
import {
  createPatchEdge,
  computeStereoPairing,
  removeEdgeFromList,
  buildAudioGraphDto,
} from "./audioGraphHelpers";

/**
 * @brief Return payload contract for useAudioGraph hook.
 */
export interface UseAudioGraphResult {
  /** @brief Current node state collection. */
  nodes: Node<AudioNodeData>[];
  /** @brief Current edge patch cords collection. */
  edges: Edge[];
  /** @brief Node state updater dispatch. */
  setNodes: React.Dispatch<React.SetStateAction<Node<AudioNodeData>[]>>;
  /** @brief Edge state updater dispatch. */
  setEdges: React.Dispatch<React.SetStateAction<Edge[]>>;
  /** @brief React Flow node changes callback. */
  onNodesChange: OnNodesChange<Node<AudioNodeData>>;
  /** @brief React Flow edge changes callback with auto-sync. */
  onEdgesChange: OnEdgesChange;
  /** @brief User pin connection callback supporting Smart Stereo Auto-Pairing. */
  onConnect: (connection: Connection) => void;
  /** @brief Volume fader change handler with debounced IPC dispatch. */
  handleVolumeChange: (id: string, volume: number) => void;
  /** @brief Mute toggle handler with IPC dispatch. */
  handleMuteToggle: (id: string, muted: boolean) => void;
  /** @brief Node removal handler removing node and connected cables. */
  handleRemoveNode: (id: string) => void;
  /** @brief Explicitly disconnects and removes a patch cord by edge ID. */
  handleRemoveEdge: (edgeId: string) => void;
  /** @brief Injects a new audio source node. */
  addSourceNode: (payload: { name: string; subtext: string; iconType: AudioIconType }) => void;
  /** @brief Injects a new virtual audio device bus node. */
  addVirtualNode: (payload: { name: string; subtext: string; driverType?: string }) => void;
  /** @brief Injects a new output destination node. */
  addOutputNode: (payload: { name: string; subtext: string; iconType: AudioIconType }) => void;
  /** @brief Synchronizes the current patchbay topology to Tauri backend. */
  syncGraphToBackend: (currentNodes: Node<AudioNodeData>[], currentEdges: Edge[]) => void;
}

/**
 * @brief Comprehensive audio patchbay graph hook.
 * @details Manages node/edge lifecycles, Smart Stereo Auto-Pairing on connection, volume/mute
 * parameters, and synchronizes the audio matrix topology with the Tauri backend.
 * @param initialNodes Optional initial nodes array (defaults to INITIAL_NODES).
 * @param initialEdges Optional initial edges array (defaults to INITIAL_EDGES).
 * @return Hook state and control methods.
 */
export function useAudioGraph(
  initialNodes: Node<AudioNodeData>[] = INITIAL_NODES,
  initialEdges: Edge[] = INITIAL_EDGES
): UseAudioGraphResult {
  const [nodes, setNodes, onNodesChange] = useNodesState<Node<AudioNodeData>>(initialNodes);
  const [edges, setEdges, reactFlowOnEdgesChange] = useEdgesState(initialEdges);

  const nodesRef = useRef(nodes);
  nodesRef.current = nodes;

  const edgesRef = useRef(edges);
  edgesRef.current = edges;

  /**
   * @brief Dispatches updated graph state to Tauri backend routing matrix.
   */
  const syncGraphToBackend = useCallback(
    (currentNodes: Node<AudioNodeData>[], currentEdges: Edge[]) => {
      const payload = buildAudioGraphDto(currentNodes, currentEdges);
      invoke("update_routing_graph", { graph: payload }).catch(() => {
        // Silently ignore in browser mock preview mode
      });
    },
    []
  );

  /**
   * @brief Updates node parameter locally and sends IPC update to backend.
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
        // Silently ignore in browser mode
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
   * @brief Removes an edge by its unique ID and updates backend matrix.
   */
  const handleRemoveEdge = useCallback(
    (edgeId: string) => {
      setEdges((eds) => {
        const nextEdges = removeEdgeFromList(eds, edgeId);
        syncGraphToBackend(nodesRef.current, nextEdges);
        return nextEdges;
      });
    },
    [setEdges, syncGraphToBackend]
  );

  /**
   * @brief Handles node deletion along with its connected edges.
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

  // Helper to insert created node and synchronize matrix
  const insertNode = useCallback(
    (node: Node<AudioNodeData>) => {
      setNodes((nds) => {
        const next = [...nds, node];
        syncGraphToBackend(next, edgesRef.current);
        return next;
      });
    },
    [setNodes, syncGraphToBackend]
  );

  // Bind handlers and custom delete to edges on mount
  useEffect(() => {
    setNodes((nds) =>
      nds.map((n) => ({
        ...n,
        data: { ...n.data, ...nodeHandlers },
      }))
    );

    setEdges((eds) =>
      eds.map((e) => ({
        ...e,
        type: e.type || "glowing",
        data: {
          ...e.data,
          onDelete: handleRemoveEdge,
        },
      }))
    );
    // Run once on mount
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  /**
   * @brief Intercepts edge changes to sync removals to the backend matrix.
   */
  const onEdgesChange: OnEdgesChange = useCallback(
    (changes: EdgeChange[]) => {
      reactFlowOnEdgesChange(changes);
      const hasRemovals = changes.some((c) => c.type === "remove");
      if (hasRemovals) {
        setTimeout(() => {
          syncGraphToBackend(nodesRef.current, edgesRef.current);
        }, 0);
      }
    },
    [reactFlowOnEdgesChange, syncGraphToBackend]
  );

  /**
   * @brief Connects pins and automatically creates Right-channel twin cord if eligible.
   */
  const onConnect = useCallback(
    (params: Connection) => {
      const primaryEdge = createPatchEdge(params);
      primaryEdge.data = {
        ...primaryEdge.data,
        onDelete: handleRemoveEdge,
      };

      const twinEdge = computeStereoPairing(params, nodesRef.current, edgesRef.current);
      if (twinEdge) {
        twinEdge.data = {
          ...twinEdge.data,
          onDelete: handleRemoveEdge,
        };
      }

      setEdges((eds) => {
        const nextEdges = twinEdge ? [...eds, primaryEdge, twinEdge] : [...eds, primaryEdge];
        syncGraphToBackend(nodesRef.current, nextEdges);
        return nextEdges;
      });
    },
    [handleRemoveEdge, setEdges, syncGraphToBackend]
  );

  const addSourceNode = useCallback(
    (payload: { name: string; subtext: string; iconType: AudioIconType }) => {
      insertNode(createSourceNode(payload, nodeHandlers));
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [insertNode, handleVolumeChange, handleMuteToggle, handleRemoveNode]
  );

  const addVirtualNode = useCallback(
    (payload: { name: string; subtext: string; driverType?: string }) => {
      insertNode(createVirtualNode(payload, nodeHandlers));
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [insertNode, handleVolumeChange, handleMuteToggle, handleRemoveNode]
  );

  const addOutputNode = useCallback(
    (payload: { name: string; subtext: string; iconType: AudioIconType }) => {
      insertNode(createOutputNode(payload, nodeHandlers));
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [insertNode, handleVolumeChange, handleMuteToggle, handleRemoveNode]
  );

  return {
    nodes,
    edges,
    setNodes,
    setEdges,
    onNodesChange,
    onEdgesChange,
    onConnect,
    handleVolumeChange,
    handleMuteToggle,
    handleRemoveNode,
    handleRemoveEdge,
    addSourceNode,
    addVirtualNode,
    addOutputNode,
    syncGraphToBackend,
  };
}
