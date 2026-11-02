/**
 * @file GlowingEdge.tsx
 * @brief Custom patchbay cord edge component featuring neon Bezier glow, signal pulse, and click-to-delete.
 * @details Implements Loopback/modular synthesizer style audio cables with glowing SVG drop shadows,
 * animated dash patterns representing active audio flow, and an interactive mid-point disconnect button.
 */

import { useState, useCallback, useMemo } from "react";
import {
  BaseEdge,
  EdgeLabelRenderer,
  getBezierPath,
  useReactFlow,
  type EdgeProps,
} from "@xyflow/react";
import { X } from "lucide-react";

/**
 * @brief Default decibel threshold above which audio signal is considered active.
 */
export const ACTIVE_AUDIO_THRESHOLD_DB = -50.0;

/**
 * @brief Extracts channel pin index (0 for Left, 1 for Right) from a React Flow handle identifier.
 * @param handleId Handle identifier string (e.g., "out-0", "in-1").
 * @return Channel index (0 for Left/Mono, 1 for Right).
 */
export function getChannelIndexFromHandle(handleId: string | null | undefined): number {
  if (!handleId) {
    return 0;
  }
  if (handleId.endsWith("1")) {
    return 1;
  }
  return 0;
}

/**
 * @brief Resolves neon accent color for Left vs Right audio channels.
 * @param sourceHandle Source pin handle string.
 * @param targetHandle Target pin handle string.
 * @param defaultColor Optional color override passed in edge styles.
 * @return Hex color code for the glowing patch cord.
 */
export function determineEdgeGlowColor(
  sourceHandle: string | null | undefined,
  targetHandle: string | null | undefined,
  defaultColor?: string
): string {
  if (defaultColor) {
    return defaultColor;
  }
  const channel = getChannelIndexFromHandle(sourceHandle ?? targetHandle);
  return channel === 1 ? "#818cf8" : "#38bdf8";
}

/**
 * @brief Evaluates whether audio signal amplitude exceeds the telemetry threshold.
 * @param signalDb Peak decibel amplitude in dBFS.
 * @param thresholdDb Decibel gate threshold (defaults to -50.0 dBFS).
 * @return Whether active audio is present.
 */
export function isAudioActive(
  signalDb: number | null | undefined,
  thresholdDb: number = ACTIVE_AUDIO_THRESHOLD_DB
): boolean {
  if (signalDb === undefined || signalDb === null || !Number.isFinite(signalDb)) {
    return false;
  }
  return signalDb >= thresholdDb;
}

/**
 * @brief Computes dynamic stroke width based on signal activity.
 * @param isActive Whether active audio signal is present.
 * @param baseWidth Base resting stroke width in pixels.
 * @return Stroke width in pixels.
 */
export function calculateGlowStrokeWidth(
  isActive: boolean,
  baseWidth: number = 2
): number {
  return isActive ? baseWidth + 1 : baseWidth;
}

/**
 * @brief Custom patchbay glowing Bezier edge component for React Flow.
 * @param props Edge properties supplied by React Flow.
 * @return {React.JSX.Element} Rendered SVG glowing path and interactive disconnect controls.
 */
export function GlowingEdge({
  id,
  sourceX,
  sourceY,
  targetX,
  targetY,
  sourcePosition,
  targetPosition,
  style = {},
  sourceHandleId,
  targetHandleId,
  data,
  markerEnd,
}: EdgeProps): React.JSX.Element {
  const { setEdges } = useReactFlow();
  const [isHovered, setIsHovered] = useState(false);

  const [edgePath, labelX, labelY] = getBezierPath({
    sourceX,
    sourceY,
    sourcePosition,
    targetX,
    targetY,
    targetPosition,
  });

  const wireColor = useMemo(
    () => determineEdgeGlowColor(sourceHandleId, targetHandleId, style.stroke as string),
    [sourceHandleId, targetHandleId, style.stroke]
  );

  const isActive = useMemo(() => {
    if (typeof data?.active === "boolean") {
      return data.active;
    }
    const signalDb = data?.signalDb as number | undefined;
    return isAudioActive(signalDb);
  }, [data]);

  const strokeWidth = calculateGlowStrokeWidth(isActive, (style.strokeWidth as number) || 2);

  const handleDelete = useCallback(
    (e: React.MouseEvent) => {
      e.stopPropagation();
      const customOnDelete = data?.onDelete as ((edgeId: string) => void) | undefined;
      if (customOnDelete) {
        customOnDelete(id);
      } else {
        setEdges((edges) => edges.filter((edge) => edge.id !== id));
      }
    },
    [id, data, setEdges]
  );

  const buttonBaseClass =
    "w-5 h-5 rounded-full bg-slate-950/90 border border-slate-700/80 text-slate-400 " +
    "hover:text-rose-400 hover:border-rose-500/80 hover:bg-slate-900 transition-all " +
    "duration-150 flex items-center justify-center shadow-lg backdrop-blur-sm cursor-pointer";
  const buttonStateClass = isHovered ? "opacity-100 scale-110" : "opacity-0 scale-75";

  return (
    <>
      <g
        className="group/edge cursor-pointer"
        onMouseEnter={() => setIsHovered(true)}
        onMouseLeave={() => setIsHovered(false)}
      >
        {/* Invisible expanded hit-box for smooth hover interactions */}
        <path
          d={edgePath}
          fill="none"
          stroke="transparent"
          strokeWidth={24}
          strokeLinecap="round"
        />

        {/* Ambient neon outer blur glow */}
        <path
          d={edgePath}
          fill="none"
          stroke={wireColor}
          strokeWidth={strokeWidth * 3.5}
          strokeOpacity={isHovered ? 0.5 : isActive ? 0.35 : 0.18}
          strokeLinecap="round"
          className="transition-all duration-300 pointer-events-none"
          style={{ filter: "blur(4px)" }}
        />

        {/* Primary solid/animated cable core */}
        <BaseEdge
          id={id}
          path={edgePath}
          markerEnd={markerEnd}
          style={{
            ...style,
            stroke: wireColor,
            strokeWidth,
            strokeLinecap: "round",
            strokeDasharray: isActive ? "6 6" : undefined,
            animation: isActive ? "dynoDash 1s linear infinite" : undefined,
          }}
          className="transition-all duration-200 pointer-events-none"
        />
      </g>

      {/* Mid-point interactive disconnect pill */}
      <EdgeLabelRenderer>
        <div
          style={{
            position: "absolute",
            transform: `translate(-50%, -50%) translate(${labelX}px,${labelY}px)`,
            pointerEvents: "all",
          }}
          className="nodrag nopan"
          onMouseEnter={() => setIsHovered(true)}
          onMouseLeave={() => setIsHovered(false)}
        >
          <button
            type="button"
            onClick={handleDelete}
            className={`${buttonBaseClass} ${buttonStateClass}`}
            title="Disconnect patch cord"
            aria-label="Disconnect patch cord"
          >
            <X size={11} strokeWidth={2.6} />
          </button>
        </div>
      </EdgeLabelRenderer>
    </>
  );
}

export default GlowingEdge;
