/**
 * @file EdgeInspectorModal.tsx
 * @brief Modal dialog for inspecting and adjusting individual connection wire properties.
 * @details Allows users to adjust linear wire gain in decibels (-60 dB to +6 dB), toggle wire muting,
 * and flip the audio phase by 180 degrees (Phase Invert Ø).
 */

import { useState, useEffect } from "react";
import { X, Volume2, VolumeX } from "lucide-react";

/**
 * @brief Properties for EdgeInspectorModal component.
 */
export interface EdgeInspectorModalProps {
  /** @brief Whether the modal is open and visible. */
  isOpen: boolean;
  /** @brief Callback to close the modal. */
  onClose: () => void;
  /** @brief Unique identifier of the connection wire. */
  edgeId: string;
  /** @brief Friendly display label of the source node. */
  sourceName?: string;
  /** @brief Friendly display label of the destination node. */
  destName?: string;
  /** @brief Current linear gain (1.0 = 0 dB unity gain). */
  initialGain?: number;
  /** @brief Current wire mute status. */
  initialMuted?: boolean;
  /** @brief Current wire phase inversion status. */
  initialPhaseInvert?: boolean;
  /** @brief Callback fired when properties are modified. */
  onSave: (gain: number, muted: boolean, phaseInvert: boolean) => void;
}

/**
 * @brief Converts linear gain multiplier to decibels.
 * @param gain Linear gain scale.
 * @return Decibel representation clamped to [-60.0, +6.0].
 */
export function linearToDb(gain: number): number {
  if (gain <= 0.001) return -60.0;
  return Math.min(6.0, Math.max(-60.0, 20.0 * Math.log10(gain)));
}

/**
 * @brief Converts decibel value back to linear multiplier.
 * @param db Value in decibels.
 * @return Linear multiplier.
 */
export function dbToLinear(db: number): number {
  if (db <= -59.5) return 0.0;
  return Math.pow(10.0, db / 20.0);
}

/**
 * @brief Modal component for configuring connection wire parameters.
 * @param props Component properties.
 * @return {React.JSX.Element | null} Rendered modal dialog.
 */
export function EdgeInspectorModal({
  isOpen,
  onClose,
  sourceName = "Source",
  destName = "Destination",
  initialGain = 1.0,
  initialMuted = false,
  initialPhaseInvert = false,
  onSave,
}: EdgeInspectorModalProps): React.JSX.Element | null {
  const [gain, setGain] = useState(initialGain);
  const [muted, setMuted] = useState(initialMuted);
  const [phaseInvert, setPhaseInvert] = useState(initialPhaseInvert);

  useEffect(() => {
    setGain(initialGain);
    setMuted(initialMuted);
    setPhaseInvert(initialPhaseInvert);
  }, [initialGain, initialMuted, initialPhaseInvert, isOpen]);

  if (!isOpen) return null;

  const currentDb = linearToDb(gain);

  const handleApply = () => {
    onSave(gain, muted, phaseInvert);
    onClose();
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4">
      <div className="w-full max-w-sm rounded-xl border border-slate-700/80 bg-slate-900/95 p-5 shadow-2xl text-slate-100">
        {/* Header */}
        <div className="flex items-center justify-between pb-3 border-b border-slate-800">
          <div>
            <h2 className="text-sm font-semibold tracking-wide text-cyan-400 uppercase">
              Wire Inspector
            </h2>
            <p className="text-xs text-slate-400 mt-0.5">
              {sourceName} → {destName}
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

        {/* Content */}
        <div className="mt-4 space-y-4">
          {/* Wire Gain Slider */}
          <div>
            <div className="flex justify-between text-xs font-medium text-slate-300 mb-1.5">
              <span>Wire Gain</span>
              <span className="text-cyan-400 font-mono">
                {currentDb <= -59.5 ? "-∞ dB" : `${currentDb > 0 ? "+" : ""}${currentDb.toFixed(1)} dB`}
              </span>
            </div>
            <input
              type="range"
              min={-60}
              max={6}
              step={0.5}
              value={currentDb}
              onChange={(e) => setGain(dbToLinear(parseFloat(e.target.value)))}
              className="w-full h-1.5 bg-slate-800 rounded-lg appearance-none cursor-pointer accent-cyan-400"
            />
            {/* Presets */}
            <div className="flex justify-between text-[10px] text-slate-500 font-mono mt-1">
              <button
                type="button"
                onClick={() => setGain(dbToLinear(-12))}
                className="hover:text-cyan-400"
              >
                -12dB
              </button>
              <button
                type="button"
                onClick={() => setGain(dbToLinear(-6))}
                className="hover:text-cyan-400"
              >
                -6dB
              </button>
              <button
                type="button"
                onClick={() => setGain(dbToLinear(0))}
                className="hover:text-cyan-400 font-semibold text-slate-300"
              >
                0dB (Unity)
              </button>
              <button
                type="button"
                onClick={() => setGain(dbToLinear(3))}
                className="hover:text-cyan-400"
              >
                +3dB
              </button>
              <button
                type="button"
                onClick={() => setGain(dbToLinear(6))}
                className="hover:text-cyan-400"
              >
                +6dB
              </button>
            </div>
          </div>

          {/* Mute and Phase Invert Toggles */}
          <div className="grid grid-cols-2 gap-2 pt-2">
            <button
              type="button"
              onClick={() => setMuted(!muted)}
              className={`flex items-center justify-center gap-1.5 py-2 px-3 rounded-lg border text-xs font-medium transition-all ${
                muted
                  ? "bg-rose-500/20 border-rose-500/50 text-rose-300"
                  : "bg-slate-800/60 border-slate-700/60 text-slate-300 hover:bg-slate-800"
              }`}
            >
              {muted ? <VolumeX size={14} /> : <Volume2 size={14} />}
              {muted ? "Muted" : "Unmuted"}
            </button>

            <button
              type="button"
              onClick={() => setPhaseInvert(!phaseInvert)}
              className={`flex items-center justify-center gap-1.5 py-2 px-3 rounded-lg border text-xs font-medium transition-all ${
                phaseInvert
                  ? "bg-amber-500/20 border-amber-500/50 text-amber-300"
                  : "bg-slate-800/60 border-slate-700/60 text-slate-300 hover:bg-slate-800"
              }`}
              title="Invert audio phase by 180 degrees (-1.0x)"
            >
              <span className="font-serif font-bold text-sm">Ø</span>
              {phaseInvert ? "Phase Inverted" : "Normal Phase"}
            </button>
          </div>
        </div>

        {/* Action Buttons */}
        <div className="mt-5 flex gap-2">
          <button
            type="button"
            onClick={onClose}
            className="flex-1 py-2 rounded-lg bg-slate-800 hover:bg-slate-700 text-xs font-medium text-slate-300 transition-colors"
          >
            Cancel
          </button>
          <button
            type="button"
            onClick={handleApply}
            className="flex-1 py-2 rounded-lg bg-cyan-600 hover:bg-cyan-500 text-xs font-semibold text-white transition-colors"
          >
            Apply
          </button>
        </div>
      </div>
    </div>
  );
}

export default EdgeInspectorModal;
