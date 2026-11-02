/**
 * @file VuMeter.tsx
 * @brief Professional LED-style peak audio level meter component.
 * @details Displays real-time audio channel amplitudes mapped from -60 dBFS to 0 dBFS,
 * featuring green/yellow/red gradient transitions, segmented LED tick marks, and peak hold markers.
 */

import React, { useMemo } from "react";

/**
 * @brief Props definition for the VuMeter component.
 */
export interface VuMeterProps {
  /** @brief Left audio channel peak decibel level (-60.0 to 0.0 dBFS). */
  leftLevelDb: number;
  /** @brief Right audio channel peak decibel level (-60.0 to 0.0 dBFS). Optional for mono channels. */
  rightLevelDb?: number;
  /** @brief Whether to render numeric decibel markers and channel labels. Default is true. */
  showLabels?: boolean;
  /** @brief Orientation of the meter ("horizontal" or "vertical"). Default is "horizontal". */
  orientation?: "horizontal" | "vertical";
  /** @brief Custom height or width class for sizing flexibility. */
  className?: string;
  /** @brief Whether the meter is in a muted state (renders fully dimmed). */
  muted?: boolean;
}

/**
 * @brief Converts a decibel value (-60 to 0 dBFS) into a normalized percentage (0% to 100%).
 * @param db Decibel value between -60 and 0.
 * @return Normalized percentage clamped between 0 and 100.
 */
export function dbToPercent(db: number): number {
  if (!Number.isFinite(db) || db <= -60) {
    return 0;
  }
  if (db >= 0) {
    return 100;
  }
  return ((db + 60) / 60) * 100;
}

/**
 * @brief Single channel meter bar component.
 * @param props Channel meter properties.
 * @return {React.JSX.Element} Rendered channel bar.
 */
function ChannelBar({
  levelDb,
  label,
  muted,
}: {
  levelDb: number;
  label: string;
  muted?: boolean;
}): React.JSX.Element {
  const percent = useMemo(() => (muted ? 0 : dbToPercent(levelDb)), [levelDb, muted]);
  const isClipping = levelDb >= -0.5 && !muted;

  return (
    <div className="flex items-center gap-1.5 w-full">
      <span className="text-[10px] font-mono font-semibold text-slate-400 w-2.5 select-none">
        {label}
      </span>
      <div className="relative flex-1 h-2 bg-slate-950/90 rounded-sm border border-slate-800/80 overflow-hidden">
        {/* Color Gradient Track */}
        <div
          className="absolute inset-y-0 left-0 transition-all duration-75 ease-out rounded-sm"
          style={{
            width: `${percent}%`,
            background:
              "linear-gradient(to right, #10b981 0%, #10b981 75%, #eab308 80%, #ef4444 95%, #dc2626 100%)",
            opacity: muted ? 0.2 : 1,
          }}
        />

        {/* LED Grid Mask Effect */}
        <div
          className="absolute inset-0 pointer-events-none opacity-30"
          style={{
            backgroundImage:
              "repeating-linear-gradient(90deg, transparent, transparent 3px, rgba(0,0,0,0.8) 3px, rgba(0,0,0,0.8) 4px)",
          }}
        />

        {/* Peak Clip Indicator */}
        {isClipping && (
          <div className="absolute right-0 top-0 bottom-0 w-1 bg-red-500 shadow-[0_0_6px_#ef4444] animate-pulse" />
        )}
      </div>
    </div>
  );
}

/**
 * @brief Professional LED-style peak VU meter for stereo or mono channel monitoring.
 * @param {VuMeterProps} props Configuration options and decibel levels.
 * @return {React.JSX.Element} The rendered VU meter component.
 */
export function VuMeter({
  leftLevelDb,
  rightLevelDb,
  showLabels = true,
  className = "",
  muted = false,
}: VuMeterProps): React.JSX.Element {
  const isStereo = typeof rightLevelDb === "number";
  const displayDb = useMemo(() => {
    if (muted) return "-inf dB";
    const maxDb = isStereo ? Math.max(leftLevelDb, rightLevelDb ?? -60) : leftLevelDb;
    if (maxDb <= -59.5) return "-inf dB";
    return `${maxDb.toFixed(1)} dB`;
  }, [leftLevelDb, rightLevelDb, isStereo, muted]);

  return (
    <div
      className={`flex flex-col gap-1 p-2 rounded-lg bg-slate-900/60 border border-slate-800/50 ${className}`}
      data-testid="vu-meter"
    >
      <div className="flex items-center justify-between text-[10px] text-slate-400 font-mono">
        <span className="text-slate-500 text-[9px] uppercase tracking-wider font-medium">
          Peak Level
        </span>
        <span
          className={`font-semibold ${
            leftLevelDb >= -3 || (rightLevelDb ?? -60) >= -3
              ? "text-rose-400"
              : leftLevelDb >= -12 || (rightLevelDb ?? -60) >= -12
              ? "text-yellow-400"
              : "text-emerald-400"
          }`}
        >
          {displayDb}
        </span>
      </div>

      <div className="flex flex-col gap-1 w-full">
        <ChannelBar levelDb={leftLevelDb} label="L" muted={muted} />
        {isStereo && <ChannelBar levelDb={rightLevelDb!} label="R" muted={muted} />}
      </div>

      {showLabels && (
        <div className="flex justify-between pl-4 pr-1 text-[8px] font-mono text-slate-500 select-none">
          <span>-60</span>
          <span>-24</span>
          <span>-12</span>
          <span>-6</span>
          <span className="text-rose-400">0</span>
        </div>
      )}
    </div>
  );
}

export default VuMeter;
