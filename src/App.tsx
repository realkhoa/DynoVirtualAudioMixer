import React from "react";
import { Sliders, Activity, Radio, Cpu, Volume2 } from "lucide-react";

/**
 * @brief Top header component displaying application title and real-time telemetry.
 * @return {React.JSX.Element} The rendered header bar.
 */
function AppHeader(): React.JSX.Element {
  return (
    <header className="flex items-center justify-between px-4 py-2.5 bg-dyno-dark/80 border-b border-dyno-border">
      <div className="flex items-center gap-3">
        <div className="flex items-center justify-center w-8 h-8 rounded-lg bg-sky-500/10 border border-sky-500/30">
          <Radio className="w-4 h-4 text-sky-400" />
        </div>
        <div>
          <h1 className="text-sm font-semibold tracking-wide text-white">
            Dyno Virtual Audio Mixer
          </h1>
          <p className="text-[11px] text-slate-400">
            Low-Latency WASAPI Audio Patchbay
          </p>
        </div>
      </div>

      <div className="flex items-center gap-3 text-xs">
        <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-md bg-slate-800/80 border border-slate-700/60">
          <Cpu className="w-3.5 h-3.5 text-sky-400" />
          <span className="text-slate-400">Sample Rate:</span>
          <span className="font-mono font-medium text-slate-200">48,000 Hz</span>
        </div>

        <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-md bg-slate-800/80 border border-slate-700/60">
          <Activity className="w-3.5 h-3.5 text-emerald-400" />
          <span className="text-slate-400">Latency:</span>
          <span className="font-mono font-medium text-emerald-400">&lt; 10 ms</span>
        </div>

        <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-md bg-slate-800/80 border border-slate-700/60">
          <span className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse" />
          <span className="text-emerald-400 font-medium">Engine Active</span>
        </div>
      </div>
    </header>
  );
}

/**
 * @brief Center workspace component displaying patchbay placeholder and master monitor.
 * @return {React.JSX.Element} The rendered canvas placeholder.
 */
function WorkspacePlaceholder(): React.JSX.Element {
  return (
    <main className="flex-1 relative flex items-center justify-center bg-radial from-slate-900/60 to-dyno-darker">
      <div className="flex flex-col items-center justify-center p-8 max-w-md text-center rounded-2xl bg-dyno-card/40">
        <div className="w-14 h-14 rounded-2xl bg-sky-500/10 border border-sky-500/20 flex items-center justify-center">
          <Sliders className="w-7 h-7 text-sky-400" />
        </div>
        <h2 className="text-lg font-medium text-white mb-1 mt-3">
          Audio Patchbay Workspace
        </h2>
        <p className="text-xs text-slate-400 mb-6 leading-relaxed">
          Drag audio source nodes, pass-through processors, and hardware output endpoints.
          Connect glowing patch cables to direct real-time audio streams.
        </p>

        <div
          className="w-full flex items-center justify-between p-3 rounded-lg bg-slate-900/80 border border-slate-800"
        >
          <div className="flex items-center gap-2 text-slate-300 text-xs">
            <Volume2 className="w-4 h-4 text-sky-400" />
            <span>Master Monitor</span>
          </div>
          <div className="font-mono text-[11px] text-emerald-400">
            -12.4 dB
          </div>
        </div>
      </div>
    </main>
  );
}

/**
 * @brief Footer status bar rendering engine version and buffer telemetry.
 * @return {React.JSX.Element} The rendered status footer.
 */
function AppFooter(): React.JSX.Element {
  return (
    <footer className="flex items-center justify-between px-4 py-1.5 bg-dyno-dark/90 border-t border-dyno-border">
      <div className="flex items-center gap-2 text-[11px] text-slate-400">
        <span>Dyno Core v0.1.0</span>
        <span>•</span>
        <span>Tauri v2 + Rust WASAPI</span>
      </div>
      <div className="flex items-center gap-3 text-[11px] text-slate-400">
        <span>Buffers: Lock-Free SPSC</span>
        <span>•</span>
        <span>Channels: Stereo / Multi-ch</span>
      </div>
    </footer>
  );
}

/**
 * @brief Root component rendering the Dyno Virtual Audio Mixer main shell.
 * @details Assembles the top telemetry bar, center node workspace, and bottom status bar.
 * @return {React.JSX.Element} The rendered root application shell.
 */
export default function App(): React.JSX.Element {
  return (
    <div className="flex flex-col h-screen w-screen bg-dyno-darker text-slate-100 select-none overflow-hidden">
      <AppHeader />
      <WorkspacePlaceholder />
      <AppFooter />
    </div>
  );
}
