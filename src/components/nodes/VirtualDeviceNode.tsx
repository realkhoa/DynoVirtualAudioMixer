/**
 * @file VirtualDeviceNode.tsx
 * @brief Custom React Flow node component representing a virtual audio cable or loopback bus.
 * @details Represents virtual audio endpoints (such as "Stream Mix", "Podcast Aux", or "Discord Bus").
 * Provides dynamic multi-channel pass-through input pins on the left, pass-through output pins on the right,
 * volume fader slider, mute toggle, channel count badge, and real-time pass-through VU meter.
 */

import React from "react";
import { Handle, Position, type NodeProps, type Node } from "@xyflow/react";
import { Layers, Volume2, VolumeX, X, Cpu } from "lucide-react";
import type { AudioNodeData } from "../../types/audio";
import { VuMeter } from "./VuMeter";

/**
 * @brief Virtual Device Node component for the React Flow audio patchbay canvas.
 * @param props React Flow node properties containing AudioNodeData payload.
 * @return {React.JSX.Element} Rendered virtual device node card.
 */
export function VirtualDeviceNode(props: NodeProps<Node<AudioNodeData>>): React.JSX.Element {
  const { id, data } = props;
  const {
    name,
    subtext,
    volume = 1.0,
    muted = false,
    inputChannels = 2,
    outputChannels = 2,
    leftLevelDb = -60,
    rightLevelDb = -60,
    onVolumeChange,
    onMuteToggle,
    onRemove,
  } = data;

  const volumePercent = Math.round(volume * 100);
  const inCount = Math.max(1, inputChannels);
  const outCount = Math.max(1, outputChannels);

  /**
   * @brief Handles slider change event and dispatches volume update.
   * @param e Range input change event.
   */
  const handleVolumeInput = (e: React.ChangeEvent<HTMLInputElement>) => {
    const newVol = parseFloat(e.target.value);
    onVolumeChange?.(id, newVol);
  };

  /**
   * @brief Toggles mute state and notifies parent graph.
   */
  const handleMuteClick = () => {
    onMuteToggle?.(id, !muted);
  };

  /**
   * @brief Dispatches node removal.
   */
  const handleRemoveClick = () => {
    onRemove?.(id);
  };

  return (
    <div className="group relative w-68 rounded-xl bg-slate-900/95 border border-slate-700/60 shadow-xl backdrop-blur-md hover:border-violet-500/50 transition-all select-none">
      {/* Left Dynamic Multi-Channel Input Handles (Pins) */}
      {Array.from({ length: inCount }).map((_, idx) => {
        const topPercent = Math.round(((idx + 1) / (inCount + 1)) * 100);
        const label = inCount === 2 ? (idx === 0 ? "L" : "R") : `${idx + 1}`;
        return (
          <div
            key={`vdev-in-${idx}`}
            className="absolute -left-3 flex items-center"
            style={{ top: `${topPercent}%` }}
          >
            <Handle
              type="target"
              position={Position.Left}
              id={`in-${idx}`}
              className="!w-3 !h-3 !bg-violet-500 !border-2 !border-slate-900 !rounded-full hover:!bg-violet-400 transition-colors cursor-crosshair"
              title={`Input Channel ${idx + 1} (Pin ${idx})`}
            />
            <span className="ml-1 text-[9px] font-mono font-bold text-violet-400 select-none">
              {label}
            </span>
          </div>
        );
      })}

      {/* Node Header */}
      <div className="flex items-center justify-between px-3.5 py-2.5 border-b border-slate-800/80 bg-slate-800/30 rounded-t-xl">
        <div className="flex items-center gap-2.5 min-w-0">
          <div className="flex items-center justify-center w-7 h-7 rounded-lg bg-violet-500/10 border border-violet-500/20 shrink-0">
            <Layers className="w-4 h-4 text-violet-400" />
          </div>
          <div className="min-w-0">
            <div className="flex items-center gap-1.5">
              <h3 className="text-xs font-semibold text-slate-100 truncate" title={name}>
                {name}
              </h3>
              {(subtext?.toLowerCase().includes("vb-audio") ||
                subtext?.toLowerCase().includes("cable") ||
                subtext?.toLowerCase().includes("windows")) && (
                <span className="px-1 py-0.2 rounded text-[8px] font-mono font-medium bg-emerald-500/20 border border-emerald-500/40 text-emerald-300" title="Windows Audio Endpoint (Visible in Windows Sound Settings)">
                  WIN
                </span>
              )}
              {inCount > 2 && (
                <span className="px-1 py-0.2 rounded text-[8px] font-mono font-semibold bg-violet-500/20 border border-violet-500/40 text-violet-300">
                  {inCount} CH
                </span>
              )}
            </div>
            <p className="text-[10px] text-slate-400 truncate flex items-center gap-1">
              <Cpu className="w-2.5 h-2.5 text-violet-400 inline" />
              {subtext || "Virtual Cable Bus"}
            </p>
          </div>
        </div>

        {onRemove && (
          <button
            type="button"
            onClick={handleRemoveClick}
            className="p-1 rounded text-slate-500 hover:text-rose-400 hover:bg-rose-500/10 transition-colors cursor-pointer"
            title="Remove virtual device node"
          >
            <X className="w-3.5 h-3.5" />
          </button>
        )}
      </div>

      {/* Node Body */}
      <div className="p-3 flex flex-col gap-3">
        {/* Real-time Pass-Through VU Meter */}
        <VuMeter
          leftLevelDb={leftLevelDb}
          rightLevelDb={rightLevelDb}
          muted={muted}
        />

        {/* Volume & Mute Controls */}
        <div className="flex items-center gap-2 pt-0.5">
          <button
            type="button"
            onClick={handleMuteClick}
            className={`p-1.5 rounded-lg border transition-all cursor-pointer ${
              muted
                ? "bg-rose-500/20 border-rose-500/40 text-rose-400"
                : "bg-slate-800/80 border-slate-700/60 text-slate-300 hover:text-violet-400 hover:border-violet-500/40"
            }`}
            title={muted ? "Unmute node" : "Mute node"}
          >
            {muted ? <VolumeX className="w-3.5 h-3.5" /> : <Volume2 className="w-3.5 h-3.5" />}
          </button>

          <div className="flex-1 flex items-center gap-2">
            <input
              type="range"
              min="0"
              max="1.5"
              step="0.01"
              value={volume}
              onChange={handleVolumeInput}
              disabled={muted}
              className="w-full h-1.5 bg-slate-800 rounded-lg appearance-none cursor-pointer accent-violet-400 disabled:opacity-40"
              title={`Volume: ${volumePercent}%`}
            />
            <span className="font-mono text-[10px] text-slate-400 w-8 text-right font-medium">
              {volumePercent}%
            </span>
          </div>
        </div>
      </div>

      {/* Right Dynamic Multi-Channel Output Handles (Pins) */}
      {Array.from({ length: outCount }).map((_, idx) => {
        const topPercent = Math.round(((idx + 1) / (outCount + 1)) * 100);
        const label = outCount === 2 ? (idx === 0 ? "L" : "R") : `${idx + 1}`;
        return (
          <div
            key={`vdev-out-${idx}`}
            className="absolute -right-3 flex items-center"
            style={{ top: `${topPercent}%` }}
          >
            <span className="mr-1 text-[9px] font-mono font-bold text-violet-400 select-none">
              {label}
            </span>
            <Handle
              type="source"
              position={Position.Right}
              id={`out-${idx}`}
              className="!w-3 !h-3 !bg-violet-500 !border-2 !border-slate-900 !rounded-full hover:!bg-violet-400 transition-colors cursor-crosshair"
              title={`Output Pass-through ${idx + 1} (Pin ${idx})`}
            />
          </div>
        );
      })}
    </div>
  );
}

export default VirtualDeviceNode;
