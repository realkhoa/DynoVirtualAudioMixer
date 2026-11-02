/**
 * @file SourceNode.tsx
 * @brief Custom React Flow node component representing an audio input source.
 * @details Represents applications (Spotify, Discord, games) or physical capture devices (microphones).
 * Provides L/R stereo output connection pins, volume fader slider, mute toggle, and real-time VU meter.
 */

import React from "react";
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
    <div className="group relative w-64 rounded-xl bg-slate-900/95 border border-slate-700/60 shadow-xl backdrop-blur-md hover:border-sky-500/50 transition-all select-none">
      {/* Node Header */}
      <div className="flex items-center justify-between px-3.5 py-2.5 border-b border-slate-800/80 bg-slate-800/30 rounded-t-xl">
        <div className="flex items-center gap-2.5 min-w-0">
          <div className="flex items-center justify-center w-7 h-7 rounded-lg bg-slate-800/80 border border-slate-700/60 shrink-0">
            {renderSourceIcon(iconType)}
          </div>
          <div className="min-w-0">
            <h3 className="text-xs font-semibold text-slate-100 truncate" title={name}>
              {name}
            </h3>
            <p className="text-[10px] text-slate-400 truncate flex items-center gap-1">
              <AudioWaveform className="w-2.5 h-2.5 text-sky-400 inline" />
              {subtext || "Audio Source"}
            </p>
          </div>
        </div>

        {onRemove && (
          <button
            type="button"
            onClick={handleRemoveClick}
            className="p-1 rounded text-slate-500 hover:text-rose-400 hover:bg-rose-500/10 transition-colors"
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
              disabled={muted}
              className="w-full h-1.5 bg-slate-800 rounded-lg appearance-none cursor-pointer accent-sky-400 disabled:opacity-40"
              title={`Volume: ${volumePercent}%`}
            />
            <span className="font-mono text-[10px] text-slate-400 w-8 text-right font-medium">
              {volumePercent}%
            </span>
          </div>
        </div>
      </div>

      {/* Right Stereo Output Handles (Pins) */}
      <div className="absolute -right-3 top-[36%] flex items-center">
        <span className="mr-1 text-[9px] font-mono font-bold text-sky-400 select-none">
          L
        </span>
        <Handle
          type="source"
          position={Position.Right}
          id="out-0"
          className="!w-3 !h-3 !bg-sky-500 !border-2 !border-slate-900 !rounded-full hover:!bg-sky-400 transition-colors cursor-crosshair"
          title="Left Output Channel (Pin 0)"
        />
      </div>

      <div className="absolute -right-3 top-[64%] flex items-center">
        <span className="mr-1 text-[9px] font-mono font-bold text-sky-400 select-none">
          R
        </span>
        <Handle
          type="source"
          position={Position.Right}
          id="out-1"
          className="!w-3 !h-3 !bg-sky-500 !border-2 !border-slate-900 !rounded-full hover:!bg-sky-400 transition-colors cursor-crosshair"
          title="Right Output Channel (Pin 1)"
        />
      </div>
    </div>
  );
}

export default SourceNode;
