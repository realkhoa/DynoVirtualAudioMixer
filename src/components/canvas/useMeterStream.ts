/**
 * @file useMeterStream.ts
 * @brief React hook subscribing to 60 FPS peak decibel telemetry from Tauri backend or fallback oscillator.
 */

import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import type { Node } from "@xyflow/react";
import type { AudioNodeData, MeterUpdateDto } from "../../types/audio";

/**
 * @brief Subscribes to 60 FPS peak audio levels and updates node meters in place.
 * @param setNodes React Flow state setter function for nodes.
 */
export function useMeterStream(
  setNodes: React.Dispatch<React.SetStateAction<Node<AudioNodeData>[]>>
): void {
  useEffect(() => {
    let isUnmounted = false;
    let unsubscribe: (() => void) | undefined;

    listen<MeterUpdateDto>("audio-meter-tick", (event) => {
      if (isUnmounted) return;
      const meterMap = new Map<string, { left: number; right: number }>();
      for (const item of event.payload.meters) {
        meterMap.set(item.node_id, { left: item.peak_left, right: item.peak_right });
      }

      setNodes((nds) =>
        nds.map((n) => {
          const reading = meterMap.get(n.id);
          if (reading) {
            return {
              ...n,
              data: { ...n.data, leftLevelDb: reading.left, rightLevelDb: reading.right },
            };
          }
          return n;
        })
      );
    })
      .then((unlistenFn) => {
        unsubscribe = unlistenFn;
      })
      .catch(() => {
        const interval = setInterval(() => {
          if (isUnmounted) return;
          setNodes((nds) =>
            nds.map((n) => {
              if (n.data.muted) {
                return { ...n, data: { ...n.data, leftLevelDb: -60, rightLevelDb: -60 } };
              }

              const baseDb = n.data.nodeType === "source" ? -16 : -18;
              const jitterL = Math.sin(Date.now() / 320 + n.id.length) * 8;
              const jitterR = Math.cos(Date.now() / 360 + n.id.length) * 8;
              const leftDb = Math.min(0, Math.max(-60, baseDb + jitterL));
              const rightDb = Math.min(0, Math.max(-60, baseDb + jitterR));

              return {
                ...n,
                data: { ...n.data, leftLevelDb: leftDb, rightLevelDb: rightDb },
              };
            })
          );
        }, 60);

        unsubscribe = () => clearInterval(interval);
      });

    return () => {
      isUnmounted = true;
      if (unsubscribe) unsubscribe();
    };
  }, [setNodes]);
}

export default useMeterStream;
