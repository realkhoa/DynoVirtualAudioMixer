/**
 * @file SourceNode.tsx
 * @brief Custom React Flow node component representing an audio input source.
 * @details Represents applications (Spotify, Discord, games) or physical capture devices (microphones).
 * Provides dynamic multi-channel output connection pins, integrated DSP noise gate controls, volume fader slider,
 * mute toggle, offline hardware status badge, and real-time VU meter.
 */

import React, { useState } from "react";
import { Handle, Position, type NodeProps, type Node } from "@xyflow/react";
import {
  Mic,
  Music,
  Radio,
  Globe,
  Gamepad2,
  Volume2,
  VolumeX,
  X,
  AudioWaveform,
  AlertCircle,
  SlidersHorizontal,
} from "lucide-react";
import type { AudioNodeData, AudioIconType } from "../../types/audio";
import { VuMeter } from "./VuMeter";

/**
 * @brief Renders the visual icon corresponding to the source node type.
 * @param iconType Classified icon identifier.
 * @return {React.JSX.Element} Lucide icon element.
 */
function renderSourceIcon(iconType: AudioIconType = "default"): React.JSX.Element {
  switch (iconType) {
    case "mic":
      return <Mic className="w-4 h-4 text-emerald-400" />;
    case "music":
      return <Music className="w-4 h-4 text-sky-400" />;
    case "browser":
      return <Globe className="w-4 h-4 text-amber-400" />;
    case "game":
      return <Gamepad2 className="w-4 h-4 text-purple-400" />;
    default:
      return <Radio className="w-4 h-4 text-sky-400" />;
  }
}

/**
 * @brief Source Node component for the React Flow audio patchbay canvas.
 * @param props React Flow node properties containing AudioNodeData payload.
 * @return {React.JSX.Element} Rendered source node card.
 */
export function SourceNode(props: NodeProps<Node<AudioNodeData>>): React.JSX.Element {
  const { id, data } = props;
  const {
    name,
    subtext,
    iconType,
    volume = 1.0,
    muted = false,
    outputChannels = 2,
    leftLevelDb = -60,
    rightLevelDb = -60,
    isOffline = false,
    noiseGateEnabled = false,
    noiseGateThresholdDb = -45.0,
    onVolumeChange,
    onMuteToggle,
    onRemove,
  } = data;

  const [gateEnabled, setGateEnabled] = useState(noiseGateEnabled);
  const [gateThreshold, setGateThreshold] = useState(noiseGateThresholdDb);
  const [showGateSettings, setShowGateSettings] = useState(false);

  const volumePercent = Math.round(volume * 100);
  const channelCount = Math.max(1, outputChannels);
  const isMic = iconType === "mic";

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
   * @brief Updates noise gate parameters in node data.
   * @param enabled Whether noise gate is active.
   * @param thresh Threshold level in dBFS.
   */
  const handleGateChange = (enabled: boolean, thresh: number) => {
    setGateEnabled(enabled);
    setGateThreshold(thresh);
    data.noiseGateEnabled = enabled;
    data.noiseGateThresholdDb = thresh;
  };

  return (
    <div
      className={`group relative w-68 rounded-xl bg-slate-900/95 border shadow-xl backdrop-blur-md transition-all select-none ${
        isOffline
          ? "border-amber-500/60 bg-amber-950/20 shadow-amber-950/40"
          : "border-slate-700/60 hover:border-sky-500/50"
      }`}
    >
      {/* Node Header */}
      <div className="flex items-center justify-between px-3.5 py-2.5 border-b border-slate-800/80 bg-slate-800/30 rounded-t-xl">
        <div className="flex items-center gap-2.5 min-w-0">
          <div className="flex items-center justify-center w-7 h-7 rounded-lg bg-slate-800/80 border border-slate-700/60 shrink-0">
            {renderSourceIcon(iconType)}
          </div>
          <div className="min-w-0">
            <div className="flex items-center gap-1.5">
              <h3 className="text-xs font-semibold text-slate-100 truncate" title={name}>
                {name}
              </h3>
              {gateEnabled && (
                <span className="px-1 py-0.2 rounded text-[8px] font-mono font-semibold bg-emerald-500/20 border border-emerald-500/40 text-emerald-300">
                  GATE
                </span>
              )}
              {isOffline && (
                <span className="px-1 py-0.2 rounded text-[8px] font-mono font-semibold bg-amber-500/20 border border-amber-500/40 text-amber-300 flex items-center gap-0.5">
                  <AlertCircle className="w-2.5 h-2.5" />
                  OFFLINE
                </span>
              )}
            </div>
            <p className="text-[10px] text-slate-400 truncate flex items-center gap-1">
              <AudioWaveform className="w-2.5 h-2.5 text-sky-400 inline" />
              {isOffline ? "Hardware Disconnected" : subtext || "Audio Source"}
            </p>
          </div>
        </div>

        {onRemove && (
          <button
            type="button"
            onClick={handleRemoveClick}
            className="p-1 rounded text-slate-500 hover:text-rose-400 hover:bg-rose-500/10 transition-colors cursor-pointer"
            title="Remove source node"
          >
            <X className="w-3.5 h-3.5" />
          </button>
        )}
      </div>

      {/* Node Body */}
      <div className="p-3 flex flex-col gap-3">
        {/* Real-time Peak VU Meter */}
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
                : "bg-slate-800/80 border-slate-700/60 text-slate-300 hover:text-sky-400 hover:border-sky-500/40"
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
              className="w-full h-1.5 bg-slate-800 rounded-lg appearance-none cursor-pointer accent-sky-400 disabled:opacity-40"
              title={`Volume: ${volumePercent}%`}
            />
            <span className="font-mono text-[10px] text-slate-400 w-8 text-right font-medium">
              {volumePercent}%
            </span>
          </div>

          {/* Noise Gate button for Mic inputs */}
          {isMic && (
            <button
              type="button"
              onClick={() => setShowGateSettings(!showGateSettings)}
              className={`p-1.5 rounded-lg border text-[10px] font-mono transition-all cursor-pointer ${
                gateEnabled
                  ? "bg-emerald-500/20 border-emerald-500/40 text-emerald-300"
                  : "bg-slate-800/80 border-slate-700/60 text-slate-400 hover:text-slate-200"
              }`}
              title="Configure DSP Noise Gate Filter"
            >
              <SlidersHorizontal className="w-3.5 h-3.5" />
            </button>
          )}
        </div>

        {/* Expanded Noise Gate Settings Strip */}
        {showGateSettings && (
          <div className="pt-2 border-t border-slate-800/60 flex flex-col gap-1.5 text-[10px] text-slate-300">
            <div className="flex items-center justify-between">
              <label className="flex items-center gap-1.5 cursor-pointer">
                <input
                  type="checkbox"
                  checked={gateEnabled}
                  onChange={(e) => handleGateChange(e.target.checked, gateThreshold)}
                  className="rounded bg-slate-800 border-slate-700 text-emerald-500 focus:ring-0"
                />
                <span className="font-medium text-emerald-400">DSP Noise Gate</span>
              </label>
              <span className="font-mono text-slate-400">{gateThreshold} dBFS</span>
            </div>
            <input
              type="range"
              min="-60"
              max="-20"
              step="1"
              value={gateThreshold}
              onChange={(e) => handleGateChange(gateEnabled, parseFloat(e.target.value))}
              disabled={!gateEnabled}
              className="w-full h-1 bg-slate-800 rounded-lg appearance-none cursor-pointer accent-emerald-400 disabled:opacity-40"
              title={`Gate Threshold: ${gateThreshold} dBFS`}
            />
          </div>
        )}
      </div>

      {/* Dynamic Multi-Channel Output Handles (Pins) */}
      {Array.from({ length: channelCount }).map((_, idx) => {
        const topPercent = Math.round(((idx + 1) / (channelCount + 1)) * 100);
        const label = channelCount === 2 ? (idx === 0 ? "L" : "R") : `${idx + 1}`;
        return (
          <div
            key={`pin-out-${idx}`}
            className="absolute -right-3 flex items-center"
            style={{ top: `${topPercent}%` }}
          >
            <span className="mr-1 text-[9px] font-mono font-bold text-sky-400 select-none">
              {label}
            </span>
            <Handle
              type="source"
              position={Position.Right}
              id={`out-${idx}`}
              className="!w-3 !h-3 !bg-sky-500 !border-2 !border-slate-900 !rounded-full hover:!bg-sky-400 transition-colors cursor-crosshair"
              title={`Output Channel ${idx + 1} (Pin ${idx})`}
            />
          </div>
        );
      })}
    </div>
  );
}

export default SourceNode;
