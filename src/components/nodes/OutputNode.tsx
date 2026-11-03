/**
 * @file OutputNode.tsx
 * @brief Custom React Flow node component representing an audio output destination.
 * @details Represents physical audio playback endpoints such as speakers, headphones, or recording interfaces.
 * Provides dynamic multi-channel input connection pins, dedicated monitor controls, volume fader slider,
 * mute toggle, offline hardware status badge, and real-time playback VU meter.
 */

import React, { useState } from "react";
import { Handle, Position, type NodeProps, type Node } from "@xyflow/react";
import { Headphones, Volume2, VolumeX, X, Speaker, AlertCircle, ShieldAlert } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import type { AudioNodeData, AudioIconType } from "../../types/audio";
import { VuMeter } from "./VuMeter";

/**
 * @brief Renders the visual icon corresponding to the output node type.
 * @param iconType Classified icon identifier.
 * @return {React.JSX.Element} Lucide icon element.
 */
function renderOutputIcon(iconType: AudioIconType = "default"): React.JSX.Element {
  switch (iconType) {
    case "headphones":
      return <Headphones className="w-4 h-4 text-emerald-400" />;
    default:
      return <Speaker className="w-4 h-4 text-emerald-400" />;
  }
}

/**
 * @brief Output Node component for the React Flow audio patchbay canvas.
 * @param props React Flow node properties containing AudioNodeData payload.
 * @return {React.JSX.Element} Rendered output node card.
 */
export function OutputNode(props: NodeProps<Node<AudioNodeData>>): React.JSX.Element {
  const { id, data } = props;
  const {
    name,
    subtext,
    iconType,
    volume = 1.0,
    muted = false,
    inputChannels = 2,
    leftLevelDb = -60,
    rightLevelDb = -60,
    isMonitor = false,
    muteWhenMonitoring = false,
    isOffline = false,
    onVolumeChange,
    onMuteToggle,
    onRemove,
  } = data;

  const [localMuteWhenMon, setLocalMuteWhenMon] = useState(muteWhenMonitoring);
  const [localIsMonitor, setLocalIsMonitor] = useState(isMonitor);

  const volumePercent = Math.round(volume * 100);
  const channelCount = Math.max(1, inputChannels);

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

  /**
   * @brief Updates monitor mode configuration and syncs with backend.
   * @param mon Whether this output is a dedicated monitor.
   * @param muteWhenMon Whether to mute original signal when monitor is active.
   */
  const handleMonitorToggle = (mon: boolean, muteWhenMon: boolean) => {
    setLocalIsMonitor(mon);
    setLocalMuteWhenMon(muteWhenMon);
    data.isMonitor = mon;
    data.muteWhenMonitoring = muteWhenMon;
    invoke("set_node_monitor_mode", {
      nodeId: id,
      isMonitor: mon,
      muteWhenMonitoring: muteWhenMon,
    }).catch(() => {
      // In browser preview mode, state is saved locally in data
    });
  };

  return (
    <div
      className={`group relative w-68 rounded-xl bg-slate-900/95 border shadow-xl backdrop-blur-md transition-all select-none ${
        isOffline
          ? "border-amber-500/60 bg-amber-950/20 shadow-amber-950/40"
          : "border-slate-700/60 hover:border-emerald-500/50"
      }`}
    >
      {/* Dynamic Multi-Channel Input Handles (Pins) */}
      {Array.from({ length: channelCount }).map((_, idx) => {
        const topPercent = Math.round(((idx + 1) / (channelCount + 1)) * 100);
        const label = channelCount === 2 ? (idx === 0 ? "L" : "R") : `${idx + 1}`;
        return (
          <div
            key={`pin-in-${idx}`}
            className="absolute -left-3 flex items-center"
            style={{ top: `${topPercent}%` }}
          >
            <Handle
              type="target"
              position={Position.Left}
              id={`in-${idx}`}
              className="!w-3 !h-3 !bg-emerald-500 !border-2 !border-slate-900 !rounded-full hover:!bg-emerald-400 transition-colors cursor-crosshair"
              title={`Input Channel ${idx + 1} (Pin ${idx})`}
            />
            <span className="ml-1 text-[9px] font-mono font-bold text-emerald-400 select-none">
              {label}
            </span>
          </div>
        );
      })}

      {/* Node Header */}
      <div className="flex items-center justify-between px-3.5 py-2.5 border-b border-slate-800/80 bg-slate-800/30 rounded-t-xl">
        <div className="flex items-center gap-2.5 min-w-0">
          <div className="flex items-center justify-center w-7 h-7 rounded-lg bg-emerald-500/10 border border-emerald-500/20 shrink-0">
            {renderOutputIcon(iconType)}
          </div>
          <div className="min-w-0">
            <div className="flex items-center gap-1.5">
              <h3 className="text-xs font-semibold text-slate-100 truncate" title={name}>
                {name}
              </h3>
              {localIsMonitor && (
                <span className="px-1 py-0.2 rounded text-[8px] font-mono font-semibold bg-cyan-500/20 border border-cyan-500/40 text-cyan-300">
                  MONITOR
                </span>
              )}
              {isOffline && (
                <span className="px-1 py-0.2 rounded text-[8px] font-mono font-semibold bg-amber-500/20 border border-amber-500/40 text-amber-300 flex items-center gap-0.5">
                  <AlertCircle className="w-2.5 h-2.5" />
                  OFFLINE
                </span>
              )}
            </div>
            <p className="text-[10px] text-slate-400 truncate">
              {isOffline ? "Device unplugged • Routing to fallback" : subtext || "Playback Endpoint"}
            </p>
          </div>
        </div>

        {onRemove && (
          <button
            type="button"
            onClick={handleRemoveClick}
            className="p-1 rounded text-slate-500 hover:text-rose-400 hover:bg-rose-500/10 transition-colors cursor-pointer"
            title="Remove output node"
          >
            <X className="w-3.5 h-3.5" />
          </button>
        )}
      </div>

      {/* Node Body */}
      <div className="p-3 flex flex-col gap-3">
        {/* Real-time Playback VU Meter */}
        <VuMeter
          leftLevelDb={leftLevelDb}
          rightLevelDb={rightLevelDb}
          muted={muted || isOffline}
        />

        {/* Volume & Mute Controls */}
        <div className="flex items-center gap-2 pt-0.5">
          <button
            type="button"
            onClick={handleMuteClick}
            disabled={isOffline}
            className={`p-1.5 rounded-lg border transition-all cursor-pointer ${
              muted
                ? "bg-rose-500/20 border-rose-500/40 text-rose-400"
                : "bg-slate-800/80 border-slate-700/60 text-slate-300 hover:text-emerald-400 hover:border-emerald-500/40"
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
              disabled={muted || isOffline}
              className="w-full h-1.5 bg-slate-800 rounded-lg appearance-none cursor-pointer accent-emerald-400 disabled:opacity-40"
              title={`Volume: ${volumePercent}%`}
            />
            <span className="font-mono text-[10px] text-slate-400 w-8 text-right font-medium">
              {volumePercent}%
            </span>
          </div>
        </div>

        {/* Monitor Mode Configuration Strip */}
        <div className="pt-1 border-t border-slate-800/60 flex items-center justify-between text-[10px] text-slate-400">
          <label className="flex items-center gap-1.5 cursor-pointer">
            <input
              type="checkbox"
              checked={localIsMonitor}
              onChange={(e) => handleMonitorToggle(e.target.checked, localMuteWhenMon)}
              className="rounded bg-slate-800 border-slate-700 text-emerald-500 focus:ring-0"
            />
            <span>Monitor Device</span>
          </label>

          {localIsMonitor && (
            <label
              className="flex items-center gap-1 cursor-pointer text-cyan-300 hover:text-cyan-200"
              title="Mute local system audio while listening through this monitor to prevent feedback"
            >
              <input
                type="checkbox"
                checked={localMuteWhenMon}
                onChange={(e) => handleMonitorToggle(localIsMonitor, e.target.checked)}
                className="rounded bg-slate-800 border-slate-700 text-cyan-500 focus:ring-0"
              />
              <span className="flex items-center gap-0.5">
                <ShieldAlert className="w-2.5 h-2.5" />
                Mute When Mon
              </span>
            </label>
          )}
        </div>
      </div>
    </div>
  );
}

export default OutputNode;
