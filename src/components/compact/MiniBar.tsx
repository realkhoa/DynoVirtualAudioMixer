/**
 * @file MiniBar.tsx
 * @brief Floating compact mini bar widget for live streaming and gaming sessions.
 * @details Provides a high-density, low-footprint audio mixer overlay with quick volume sliders,
 * mute toggles, real-time mini peak VU meters, and an expand button to restore the full graph patchbay.
 */

import React from "react";
import {
  Radio,
  Maximize2,
  Volume2,
  VolumeX,
  Mic,
  Speaker,
  Headphones,
  Music,
  Globe,
  Gamepad2,
  Layers,
  Sparkles,
} from "lucide-react";
import type { Node } from "@xyflow/react";
import type { AudioNodeData, AudioIconType } from "../../types/audio";

/**
 * @brief Props definition for the MiniBar component.
 */
export interface MiniBarProps {
  /** @brief Current collection of mixer nodes in the patchbay graph. */
  nodes: Node<AudioNodeData>[];
  /** @brief Callback to adjust linear volume of a node. */
  onVolumeChange: (id: string, volume: number) => void;
  /** @brief Callback to toggle mute state of a node. */
  onMuteToggle: (id: string, muted: boolean) => void;
  /** @brief Callback to restore the full patchbay studio graph canvas. */
  onExpandToGraph: () => void;
  /** @brief Optional callback to open the Sidechain Auto-Ducking modal. */
  onOpenDucking?: () => void;
  /** @brief Whether sidechain auto-ducking is actively configured and running. */
  isDuckingActive?: boolean;
}

/**
 * @brief Selects appropriate icon element for node classification in MiniBar.
 * @param iconType Classified icon string.
 * @param nodeType Node type category.
 * @return {React.JSX.Element} Rendered Lucide icon.
 */
function renderMiniIcon(
  iconType?: AudioIconType,
  nodeType?: string
): React.JSX.Element {
  if (nodeType === "virtual") {
    return <Layers className="w-3.5 h-3.5 text-violet-400" />;
  }
  switch (iconType) {
    case "mic":
      return <Mic className="w-3.5 h-3.5 text-emerald-400" />;
    case "headphones":
      return <Headphones className="w-3.5 h-3.5 text-emerald-400" />;
    case "music":
      return <Music className="w-3.5 h-3.5 text-sky-400" />;
    case "browser":
      return <Globe className="w-3.5 h-3.5 text-amber-400" />;
    case "game":
      return <Gamepad2 className="w-3.5 h-3.5 text-purple-400" />;
    case "speaker":
      return <Speaker className="w-3.5 h-3.5 text-emerald-400" />;
    default:
      return <Radio className="w-3.5 h-3.5 text-sky-400" />;
  }
}

/**
 * @brief Floating compact audio mixing bar for high-density live streaming workflows.
 * @param props Component properties.
 * @return {React.JSX.Element} Rendered floating mini bar.
 */
export function MiniBar({
  nodes,
  onVolumeChange,
  onMuteToggle,
  onExpandToGraph,
  onOpenDucking,
  isDuckingActive = false,
}: MiniBarProps): React.JSX.Element {
  const activeNodes = nodes.filter((n) => !n.data.isOffline);

  return (
    <aside
      aria-label="Compact Mini Bar Mixer"
      className="fixed bottom-6 left-1/2 -translate-x-1/2 z-40 w-[95%] max-w-4xl rounded-2xl bg-slate-900/95 border border-slate-700/80 shadow-[0_12px_40px_rgba(0,0,0,0.6)] backdrop-blur-xl p-4 text-slate-100 transition-all select-none"
    >
      {/* Top Header of Mini Bar */}
      <div className="flex items-center justify-between pb-3 mb-3 border-b border-slate-800">
        <div className="flex items-center gap-2.5">
          <div className="flex items-center justify-center w-6 h-6 rounded-md bg-sky-500/10 border border-sky-500/30 text-sky-400">
            <Radio className="w-3.5 h-3.5" />
          </div>
          <span className="text-xs font-bold tracking-wide text-white uppercase flex items-center gap-1.5">
            Dyno Mini Bar
            <span className="text-[9px] px-1.5 py-0.2 rounded bg-sky-500/20 text-sky-300 font-mono">
              Live
            </span>
          </span>
        </div>

        <div className="flex items-center gap-2">
          {onOpenDucking && (
            <button
              type="button"
              onClick={onOpenDucking}
              className={`flex items-center gap-1 px-2.5 py-1 rounded-lg text-[11px] font-medium border transition-all cursor-pointer ${
                isDuckingActive
                  ? "bg-amber-500/15 border-amber-500/40 text-amber-300 hover:bg-amber-500/25"
                  : "bg-slate-800/80 border-slate-700/60 text-slate-400 hover:text-slate-200"
              }`}
              title="Configure Sidechain Auto-Ducking"
            >
              <Sparkles className="w-3 h-3 text-amber-400" />
              <span>Auto-Duck</span>
            </button>
          )}

          <button
            type="button"
            onClick={onExpandToGraph}
            className="flex items-center gap-1 px-2.5 py-1 rounded-lg text-[11px] font-medium bg-sky-500/20 border border-sky-500/40 text-sky-300 hover:bg-sky-500/30 transition-all cursor-pointer"
            title="Expand to Full Studio Patchbay Graph"
          >
            <Maximize2 className="w-3 h-3" />
            <span>Expand Studio</span>
          </button>
        </div>
      </div>

      {/* Node Sliders Horizontal Strip */}
      {activeNodes.length === 0 ? (
        <div className="py-4 text-center text-xs text-slate-400">
          No active channels in the patchbay matrix.
        </div>
      ) : (
        <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-3 max-h-48 overflow-y-auto pr-1">
          {activeNodes.map((node) => {
            const data = node.data;
            const volume = data.volume ?? 1.0;
            const muted = data.muted ?? false;
            const percent = Math.round(volume * 100);
            const peak = Math.max(data.leftLevelDb ?? -60, data.rightLevelDb ?? -60);
            const peakPercent = Math.min(100, Math.max(0, ((peak + 60) / 60) * 100));

            return (
              <div
                key={node.id}
                className="flex flex-col gap-1.5 p-2.5 rounded-xl bg-slate-950/60 border border-slate-800/80 hover:border-slate-700 transition-all"
              >
                {/* Node Title & Mute */}
                <div className="flex items-center justify-between gap-1.5">
                  <div className="flex items-center gap-1.5 min-w-0">
                    {renderMiniIcon(data.iconType, data.nodeType)}
                    <span
                      className="text-xs font-semibold text-slate-200 truncate"
                      title={data.name}
                    >
                      {data.name}
                    </span>
                  </div>
                  <button
                    type="button"
                    onClick={() => onMuteToggle(node.id, !muted)}
                    className={`p-1 rounded-md border transition-all cursor-pointer ${
                      muted
                        ? "bg-rose-500/20 border-rose-500/40 text-rose-400"
                        : "bg-slate-800/80 border-slate-700/60 text-slate-400 hover:text-slate-200"
                    }`}
                    title={muted ? "Unmute" : "Mute"}
                  >
                    {muted ? (
                      <VolumeX className="w-3 h-3" />
                    ) : (
                      <Volume2 className="w-3 h-3" />
                    )}
                  </button>
                </div>

                {/* Mini Horizontal Peak Bar */}
                <div className="w-full h-1 bg-slate-800 rounded-full overflow-hidden">
                  <div
                    className={`h-full transition-all duration-75 ${
                      muted
                        ? "bg-slate-600"
                        : peak > -3
                        ? "bg-rose-400"
                        : peak > -12
                        ? "bg-amber-400"
                        : "bg-emerald-400"
                    }`}
                    style={{ width: muted ? "0%" : `${peakPercent}%` }}
                  />
                </div>

                {/* Volume Slider & Percent Readout */}
                <div className="flex items-center gap-2 pt-0.5">
                  <input
                    type="range"
                    min="0"
                    max="1.5"
                    step="0.01"
                    value={volume}
                    onChange={(e) =>
                      onVolumeChange(node.id, parseFloat(e.target.value))
                    }
                    disabled={muted}
                    className="w-full h-1 bg-slate-800 rounded-lg appearance-none cursor-pointer accent-sky-400 disabled:opacity-40"
                    title={`Volume: ${percent}%`}
                  />
                  <span className="font-mono text-[10px] text-slate-400 w-7 text-right">
                    {percent}%
                  </span>
                </div>
              </div>
            );
          })}
        </div>
      )}
    </aside>
  );
}

export default MiniBar;
