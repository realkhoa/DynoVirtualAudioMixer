/**
 * @file App.tsx
 * @brief Root application container for Dyno Virtual Audio Mixer.
 * @details Integrates the interactive TopBar patchbay action header, the React Flow node graph
 * canvas, and the real-time buffer telemetry footer.
 */

import React, { useRef } from "react";
import { TopBar, type TopBarHandle } from "./components/TopBar";
import { GraphCanvas, type GraphCanvasHandle } from "./components/GraphCanvas";

/**
 * @brief Footer status bar rendering engine version and buffer telemetry.
 * @return {React.JSX.Element} The rendered status footer.
 */
function AppFooter(): React.JSX.Element {
  return (
    <footer
      className={[
        "flex items-center justify-between px-5 py-1.5",
        "bg-slate-900/95 border-t border-slate-800",
        "text-slate-400 text-[11px] select-none z-20",
      ].join(" ")}
    >
      <div className="flex items-center gap-2">
        <span className="font-semibold text-slate-300">Dyno Core v0.1.0</span>
        <span>•</span>
        <span>Tauri v2 + Rust WASAPI Engine</span>
      </div>
      <div className="flex items-center gap-3">
        <span>Buffers: Lock-Free SPSC RingBuffer</span>
        <span>•</span>
        <span>Channels: Stereo / Multi-Channel</span>
        <span>•</span>
        <span className="text-emerald-400 font-mono">60 FPS VU Metering</span>
      </div>
    </footer>
  );
}

/**
 * @brief Root component rendering the Dyno Virtual Audio Mixer main shell.
 * @return {React.JSX.Element} The rendered root application shell.
 */
export default function App(): React.JSX.Element {
  const canvasRef = useRef<GraphCanvasHandle>(null);
  const topBarRef = useRef<TopBarHandle>(null);

  return (
    <div className="flex flex-col h-screen w-screen bg-[#0a0f1d] text-slate-100 select-none overflow-hidden">
      <TopBar
        ref={topBarRef}
        onAddSource={(p) => canvasRef.current?.addSourceNode(p)}
        onAddVirtualDevice={(p) => canvasRef.current?.addVirtualNode(p)}
        onAddOutput={(p) => canvasRef.current?.addOutputNode(p)}
        onClearGraph={() => canvasRef.current?.clearGraph()}
        onResetHardware={() => canvasRef.current?.resetToHardwareDefaults()}
      />
      <main className="flex-1 relative w-full h-full overflow-hidden">
        <GraphCanvas
          ref={canvasRef}
          onOpenAddSource={() => topBarRef.current?.openAddSource()}
          onOpenAddOutput={() => topBarRef.current?.openAddOutput()}
        />
      </main>
      <AppFooter />
    </div>
  );
}
