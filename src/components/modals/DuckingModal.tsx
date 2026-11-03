/**
 * @file DuckingModal.tsx
 * @brief Configuration modal dialog for Sidechain Auto-Ducking.
 * @details Enables streamers and podcasters to automatically lower background music/game volume
 * whenever speaking into their microphone, with configurable threshold, depth, and attack/release envelopes.
 */

import { useState } from "react";
import { X, Mic, Volume2 } from "lucide-react";
import type { AutoDuckerConfig, NodeDto } from "../../types/audio";

/**
 * @brief Properties for DuckingModal component.
 */
export interface DuckingModalProps {
  /** @brief Whether the modal is open and visible. */
  isOpen: boolean;
  /** @brief Callback to close the modal. */
  onClose: () => void;
  /** @brief Available mixer nodes in the patchbay. */
  nodes: NodeDto[];
  /** @brief Current auto-ducking configuration if previously set. */
  currentConfig?: AutoDuckerConfig | null;
  /** @brief Callback fired when the user saves the ducking configuration. */
  onSave: (config: AutoDuckerConfig) => void;
}

/**
 * @brief Sidechain auto-ducking configuration modal dialog.
 * @param props Component properties.
 * @return {React.JSX.Element | null} Rendered modal dialog.
 */
export function DuckingModal({
  isOpen,
  onClose,
  nodes,
  currentConfig,
  onSave,
}: DuckingModalProps): React.JSX.Element | null {
  const [triggerId, setTriggerId] = useState<string>(
    currentConfig?.trigger_node_id || (nodes.find((n) => n.input_channels > 0)?.id ?? "")
  );
  const [targetIds, setTargetIds] = useState<string[]>(
    currentConfig?.target_node_ids || []
  );
  const [thresholdDb, setThresholdDb] = useState<number>(
    currentConfig?.threshold_db ?? -35.0
  );
  const [attenuationDb, setAttenuationDb] = useState<number>(
    currentConfig?.duck_attenuation_db ?? -15.0
  );
  const [attackMs, setAttackMs] = useState<number>(
    currentConfig?.attack_ms ?? 15.0
  );
  const [releaseMs, setReleaseMs] = useState<number>(
    currentConfig?.release_ms ?? 400.0
  );
  const [enabled, setEnabled] = useState<boolean>(
    currentConfig?.enabled ?? true
  );

  if (!isOpen) return null;

  const toggleTarget = (id: string) => {
    setTargetIds((prev) =>
      prev.includes(id) ? prev.filter((t) => t !== id) : [...prev, id]
    );
  };

  const handleSave = () => {
    onSave({
      trigger_node_id: triggerId,
      target_node_ids: targetIds,
      threshold_db: thresholdDb,
      duck_attenuation_db: attenuationDb,
      attack_ms: attackMs,
      release_ms: releaseMs,
      enabled,
    });
    onClose();
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4">
      <div className="w-full max-w-md rounded-xl border border-slate-700/80 bg-slate-900/95 p-6 shadow-2xl text-slate-100">
        {/* Header */}
        <div className="flex items-center justify-between pb-3 border-b border-slate-800">
          <div>
            <h2 className="text-base font-semibold tracking-wide text-cyan-400">
              Auto-Ducking (Sidechain)
            </h2>
            <p className="text-xs text-slate-400 mt-0.5">
              Automatically lower music volume when voice activity is detected
            </p>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="text-slate-400 hover:text-slate-200 transition-colors"
          >
            <X size={18} />
          </button>
        </div>

        {/* Form Body */}
        <div className="mt-4 space-y-4">
          {/* Enable Toggle */}
          <div className="flex items-center justify-between p-3 rounded-lg bg-slate-800/50 border border-slate-700/50">
            <span className="text-sm font-medium text-slate-200">
              Enable Auto-Ducking
            </span>
            <input
              type="checkbox"
              checked={enabled}
              onChange={(e) => setEnabled(e.target.checked)}
              className="w-4 h-4 rounded text-cyan-500 accent-cyan-500 cursor-pointer"
            />
          </div>

          {/* Trigger Node Select */}
          <div>
            <label className="flex items-center gap-1.5 text-xs font-medium text-slate-300 mb-1.5">
              <Mic size={14} className="text-cyan-400" />
              Voice Trigger (Microphone)
            </label>
            <select
              value={triggerId}
              onChange={(e) => setTriggerId(e.target.value)}
              className="w-full bg-slate-800 border border-slate-700 rounded-lg px-3 py-2 text-xs text-slate-200 focus:outline-none focus:border-cyan-500"
            >
              {nodes.map((node) => (
                <option key={node.id} value={node.id}>
                  {node.name} ({node.id})
                </option>
              ))}
            </select>
          </div>

          {/* Target Nodes Select */}
          <div>
            <label className="flex items-center gap-1.5 text-xs font-medium text-slate-300 mb-1.5">
              <Volume2 size={14} className="text-indigo-400" />
              Target Channels to Duck (Music / Game)
            </label>
            <div className="max-h-28 overflow-y-auto space-y-1.5 p-2 bg-slate-800/40 border border-slate-700/60 rounded-lg">
              {nodes
                .filter((n) => n.id !== triggerId)
                .map((node) => (
                  <label
                    key={node.id}
                    className="flex items-center gap-2 text-xs text-slate-300 hover:text-white cursor-pointer"
                  >
                    <input
                      type="checkbox"
                      checked={targetIds.includes(node.id)}
                      onChange={() => toggleTarget(node.id)}
                      className="rounded accent-cyan-500"
                    />
                    <span>{node.name}</span>
                  </label>
                ))}
            </div>
          </div>

          {/* Threshold Slider */}
          <div>
            <div className="flex justify-between text-xs text-slate-300 mb-1">
              <span>Voice Sensitivity Threshold</span>
              <span className="font-mono text-cyan-400">{thresholdDb.toFixed(0)} dBFS</span>
            </div>
            <input
              type="range"
              min={-60}
              max={-10}
              step={1}
              value={thresholdDb}
              onChange={(e) => setThresholdDb(parseFloat(e.target.value))}
              className="w-full h-1.5 bg-slate-800 rounded-lg appearance-none cursor-pointer accent-cyan-400"
            />
          </div>

          {/* Attenuation Slider */}
          <div>
            <div className="flex justify-between text-xs text-slate-300 mb-1">
              <span>Volume Reduction</span>
              <span className="font-mono text-cyan-400">{attenuationDb.toFixed(0)} dB</span>
            </div>
            <input
              type="range"
              min={-30}
              max={-3}
              step={1}
              value={attenuationDb}
              onChange={(e) => setAttenuationDb(parseFloat(e.target.value))}
              className="w-full h-1.5 bg-slate-800 rounded-lg appearance-none cursor-pointer accent-cyan-400"
            />
          </div>

          {/* Attack & Release */}
          <div className="grid grid-cols-2 gap-3 text-xs">
            <div>
              <div className="flex justify-between text-slate-400 mb-1">
                <span>Attack</span>
                <span className="font-mono text-slate-300">{attackMs}ms</span>
              </div>
              <input
                type="range"
                min={1}
                max={50}
                step={1}
                value={attackMs}
                onChange={(e) => setAttackMs(parseFloat(e.target.value))}
                className="w-full h-1.5 bg-slate-800 rounded-lg appearance-none cursor-pointer accent-cyan-400"
              />
            </div>
            <div>
              <div className="flex justify-between text-slate-400 mb-1">
                <span>Release</span>
                <span className="font-mono text-slate-300">{releaseMs}ms</span>
              </div>
              <input
                type="range"
                min={100}
                max={1500}
                step={50}
                value={releaseMs}
                onChange={(e) => setReleaseMs(parseFloat(e.target.value))}
                className="w-full h-1.5 bg-slate-800 rounded-lg appearance-none cursor-pointer accent-cyan-400"
              />
            </div>
          </div>
        </div>

        {/* Footer */}
        <div className="mt-6 flex gap-2">
          <button
            type="button"
            onClick={onClose}
            className="flex-1 py-2 rounded-lg bg-slate-800 hover:bg-slate-700 text-xs font-medium text-slate-300 transition-colors"
          >
            Cancel
          </button>
          <button
            type="button"
            onClick={handleSave}
            className="flex-1 py-2 rounded-lg bg-cyan-600 hover:bg-cyan-500 text-xs font-semibold text-white transition-colors"
          >
            Save Configuration
          </button>
        </div>
      </div>
    </div>
  );
}

export default DuckingModal;
