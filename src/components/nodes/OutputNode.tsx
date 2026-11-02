/**
 * @file OutputNode.tsx
 * @brief Custom React Flow node component representing an audio output destination.
 * @details Represents physical audio playback endpoints such as speakers, headphones, or recording interfaces.
 * Provides L/R stereo input connection pins, volume fader slider, mute toggle, and real-time playback VU meter.
 */

import React from "react";
import { Handle, Position, type NodeProps, type Node } from "@xyflow/react";
import { Headphones, Volume2, VolumeX, X, Speaker } from "lucide-react";
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
    leftLevelDb = -60,
    rightLevelDb = -60,
    onVolumeChange,
    onMuteToggle,
    onRemove,
  } = data;

  const volumePercent = Math.round(volume * 100);

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
    <div className="group relative w-64 rounded-xl bg-slate-900/95 border border-slate-700/60 shadow-xl backdrop-blur-md hover:border-emerald-500/50 transition-all select-none">
      {/* Left Stereo Input Handles (Pins) */}
      <div className="absolute -left-3 top-[36%] flex items-center">
        <Handle
          type="target"
          position={Position.Left}
          id="in-0"
          className="!w-3 !h-3 !bg-emerald-500 !border-2 !border-slate-900 !rounded-full hover:!bg-emerald-400 transition-colors cursor-crosshair"
          title="Left Input Channel (Pin 0)"
        />
        <span className="ml-1 text-[9px] font-mono font-bold text-emerald-400 select-none">
          L
        </span>
      </div>

      <div className="absolute -left-3 top-[64%] flex items-center">
        <Handle
          type="target"
          position={Position.Left}
          id="in-1"
          className="!w-3 !h-3 !bg-emerald-500 !border-2 !border-slate-900 !rounded-full hover:!bg-emerald-400 transition-colors cursor-crosshair"
          title="Right Input Channel (Pin 1)"
        />
        <span className="ml-1 text-[9px] font-mono font-bold text-emerald-400 select-none">
          R
        </span>
      </div>

      {/* Node Header */}
      <div className="flex items-center justify-between px-3.5 py-2.5 border-b border-slate-800/80 bg-slate-800/30 rounded-t-xl">
        <div className="flex items-center gap-2.5 min-w-0">
          <div className="flex items-center justify-center w-7 h-7 rounded-lg bg-emerald-500/10 border border-emerald-500/20 shrink-0">
            {renderOutputIcon(iconType)}
          </div>
          <div className="min-w-0">
            <h3 className="text-xs font-semibold text-slate-100 truncate" title={name}>
              {name}
            </h3>
            <p className="text-[10px] text-slate-400 truncate">
              {subtext || "Playback Endpoint"}
            </p>
          </div>
        </div>

        {onRemove && (
          <button
            type="button"
            onClick={handleRemoveClick}
            className="p-1 rounded text-slate-500 hover:text-rose-400 hover:bg-rose-500/10 transition-colors"
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
          muted={muted}
        />

        {/* Volume & Mute Controls */}
        <div className="flex items-center gap-2 pt-0.5">
          <button
            type="button"
            onClick={handleMuteClick}
            className={`p-1.5 rounded-lg border transition-all ${
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
              disabled={muted}
              className="w-full h-1.5 bg-slate-800 rounded-lg appearance-none cursor-pointer accent-emerald-400 disabled:opacity-40"
              title={`Volume: ${volumePercent}%`}
            />
            <span className="font-mono text-[10px] text-slate-400 w-8 text-right font-medium">
              {volumePercent}%
            </span>
          </div>
        </div>
      </div>
    </div>
  );
}

export default OutputNode;
