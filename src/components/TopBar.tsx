/**
 * @file TopBar.tsx
 * @brief Application header bar and modal managers for adding sources, virtual buses, outputs, auto-ducking, and compact mode.
 * @details Queries active WASAPI devices, running applications, and virtual cables via Tauri IPC,
 * rendering responsive modal dialogs to inject new nodes into the React Flow patchbay graph.
 */

import React, { useState, useEffect, forwardRef, useImperativeHandle } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Radio,
  Plus,
  Trash2,
  RotateCcw,
  Cpu,
  Activity,
  Sparkles,
  Minimize2,
} from "lucide-react";
import type {
  AudioDeviceDescriptor,
  AudioProcessDescriptor,
  AudioIconType,
} from "../types/audio";
import { AddSourceModal } from "./modals/AddSourceModal";
import { CreateVirtualModal } from "./modals/CreateVirtualModal";
import { AddOutputModal } from "./modals/AddOutputModal";

/**
 * @brief Props definition for the TopBar component.
 */
export interface TopBarProps {
  /** @brief Callback to inject a new source node into the patchbay. */
  onAddSource: (payload: {
    name: string;
    subtext: string;
    iconType: AudioIconType;
    pid?: number;
    nodeType?: string;
  }) => void;
  /** @brief Callback to inject a new virtual device node into the patchbay. */
  onAddVirtualDevice: (payload: { name: string; subtext: string; driverType?: string; channels?: number }) => void;
  /** @brief Callback to inject a new output node into the patchbay. */
  onAddOutput: (payload: { name: string; subtext: string; iconType: AudioIconType }) => void;
  /** @brief Callback to clear all nodes and edges from the canvas. */
  onClearGraph?: () => void;
  /** @brief Callback to reload real hardware defaults. */
  onResetHardware?: () => void;
  /** @brief Callback to trigger Sidechain Auto-Ducking modal. */
  onOpenDucking?: () => void;
  /** @brief Callback to toggle floating Mini Bar overlay. */
  onToggleMiniBar?: () => void;
  /** @brief Whether compact mini bar is currently active. */
  isMiniBarActive?: boolean;
  /** @brief Whether auto ducking is active. */
  isDuckingActive?: boolean;
}

/**
 * @brief Imperative API exposed by TopBar to open dialogs programmatically.
 */
export interface TopBarHandle {
  openAddSource: () => void;
  openAddVirtualDevice: () => void;
  openAddOutput: () => void;
}

/**
 * @brief Top navigation bar and modal manager for patchbay graph operations.
 * @param {TopBarProps} props Callback functions for graph mutations.
 * @param {React.ForwardedRef<TopBarHandle>} ref Forwarded handle.
 * @return {React.JSX.Element} Rendered TopBar header.
 */
function TopBarRender(
  props: TopBarProps,
  ref: React.ForwardedRef<TopBarHandle>
): React.JSX.Element {
  const {
    onAddSource,
    onAddVirtualDevice,
    onAddOutput,
    onClearGraph,
    onResetHardware,
    onOpenDucking,
    onToggleMiniBar,
    isMiniBarActive = false,
    isDuckingActive = false,
  } = props;
  const [activeModal, setActiveModal] = useState<"source" | "virtual" | "output" | null>(null);
  const [apps, setApps] = useState<AudioProcessDescriptor[]>([]);
  const [devices, setDevices] = useState<AudioDeviceDescriptor[]>([]);

  useImperativeHandle(ref, () => ({
    openAddSource: () => setActiveModal("source"),
    openAddVirtualDevice: () => setActiveModal("virtual"),
    openAddOutput: () => setActiveModal("output"),
  }));

  useEffect(() => {
    let isMounted = true;

    async function loadData() {
      try {
        const runningApps = await invoke<AudioProcessDescriptor[]>("get_running_apps");
        if (isMounted) setApps(Array.isArray(runningApps) ? runningApps : []);
      } catch {
        if (isMounted) setApps([]);
      }

      try {
        const devList = await invoke<AudioDeviceDescriptor[]>("get_devices");
        if (isMounted) setDevices(Array.isArray(devList) ? devList : []);
      } catch {
        if (isMounted) setDevices([]);
      }
    }

    if (activeModal) {
      loadData();
    }

    return () => {
      isMounted = false;
    };
  }, [activeModal]);

  return (
    <>
      <header className="flex items-center justify-between px-5 py-2.5 bg-slate-900/90 border-b border-slate-800 backdrop-blur-md z-20">
        {/* Brand / Logo */}
        <div className="flex items-center gap-3">
          <div className="flex items-center justify-center w-8 h-8 rounded-lg bg-sky-500/10 border border-sky-500/30 shadow-[0_0_12px_rgba(56,189,248,0.2)]">
            <Radio className="w-4 h-4 text-sky-400" />
          </div>
          <div>
            <h1 className="text-sm font-semibold tracking-wide text-white flex items-center gap-2">
              Dyno Virtual Audio Mixer
              <span className="px-1.5 py-0.5 rounded text-[9px] font-mono font-medium bg-sky-500/10 border border-sky-500/30 text-sky-400">
                PRO
              </span>
            </h1>
            <p className="text-[11px] text-slate-400">
              Low-Latency WASAPI Audio Patchbay
            </p>
          </div>
        </div>

        {/* Action Buttons */}
        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={() => setActiveModal("source")}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium bg-sky-500/15 border border-sky-500/30 text-sky-300 hover:bg-sky-500/25 hover:border-sky-500/50 shadow-sm transition-all cursor-pointer"
          >
            <Plus className="w-3.5 h-3.5" />
            <span>Add Source</span>
          </button>

          <button
            type="button"
            onClick={() => setActiveModal("virtual")}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium bg-violet-500/15 border border-violet-500/30 text-violet-300 hover:bg-violet-500/25 hover:border-violet-500/50 shadow-sm transition-all cursor-pointer"
          >
            <Plus className="w-3.5 h-3.5" />
            <span>New Virtual Device</span>
          </button>

          <button
            type="button"
            onClick={() => setActiveModal("output")}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium bg-emerald-500/15 border border-emerald-500/30 text-emerald-300 hover:bg-emerald-500/25 hover:border-emerald-500/50 shadow-sm transition-all cursor-pointer"
          >
            <Plus className="w-3.5 h-3.5" />
            <span>Add Output</span>
          </button>

          <div className="h-4 w-[1px] bg-slate-800 mx-1" />

          {/* Sidechain Auto-Ducking Trigger */}
          {onOpenDucking && (
            <button
              type="button"
              onClick={onOpenDucking}
              title="Sidechain Auto-Ducking (Priority Mic Ducking)"
              className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium border transition-all cursor-pointer ${
                isDuckingActive
                  ? "bg-amber-500/20 border-amber-500/40 text-amber-300 shadow-[0_0_10px_rgba(245,158,11,0.2)]"
                  : "bg-slate-800/80 border-slate-700/60 text-slate-300 hover:text-amber-400 hover:border-amber-500/40"
              }`}
            >
              <Sparkles className="w-3.5 h-3.5 text-amber-400" />
              <span>Auto-Duck</span>
            </button>
          )}

          {/* Compact Mini Bar Mode Toggle */}
          {onToggleMiniBar && (
            <button
              type="button"
              onClick={onToggleMiniBar}
              title="Toggle Compact Mini Bar Overlay"
              className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium border transition-all cursor-pointer ${
                isMiniBarActive
                  ? "bg-sky-500/20 border-sky-500/40 text-sky-300 shadow-[0_0_10px_rgba(56,189,248,0.2)]"
                  : "bg-slate-800/80 border-slate-700/60 text-slate-300 hover:text-sky-400 hover:border-sky-500/40"
              }`}
            >
              <Minimize2 className="w-3.5 h-3.5" />
              <span>Mini Bar</span>
            </button>
          )}

          <div className="h-4 w-[1px] bg-slate-800 mx-1" />

          {/* Reset real hardware defaults */}
          {onResetHardware && (
            <button
              type="button"
              onClick={onResetHardware}
              title="Reset to Real Default Hardware Devices"
              className="flex items-center gap-1 px-2.5 py-1.5 rounded-lg text-xs font-medium text-slate-400 hover:text-sky-300 hover:bg-slate-800 border border-transparent hover:border-slate-700 transition-all cursor-pointer"
            >
              <RotateCcw className="w-3.5 h-3.5" />
              <span className="hidden sm:inline">Reset Defaults</span>
            </button>
          )}

          {/* Clear Graph to Empty */}
          {onClearGraph && (
            <button
              type="button"
              onClick={onClearGraph}
              title="Clear all nodes and wires"
              className="flex items-center gap-1 px-2.5 py-1.5 rounded-lg text-xs font-medium text-slate-400 hover:text-red-400 hover:bg-red-500/10 border border-transparent hover:border-red-500/20 transition-all cursor-pointer"
            >
              <Trash2 className="w-3.5 h-3.5" />
              <span className="hidden sm:inline">Clear Canvas</span>
            </button>
          )}
        </div>

        {/* Status indicator badges */}
        <div className="flex items-center gap-3">
          <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-md bg-slate-800/80 border border-slate-700/50 text-[11px] text-slate-300">
            <Cpu className="w-3 h-3 text-sky-400" />
            <span className="font-mono">48 kHz</span>
          </div>
          <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-md bg-emerald-500/10 border border-emerald-500/30 text-[11px] text-emerald-400">
            <Activity className="w-3 h-3 animate-pulse" />
            <span className="font-medium">Active</span>
          </div>
        </div>
      </header>

      {/* Modal: Add Source */}
      {activeModal === "source" && (
        <AddSourceModal
          apps={apps}
          devices={devices}
          onSelectApp={(app, iconType) => {
            onAddSource({
              name: app.title || app.name,
              subtext: `PID: ${app.pid} • Process Audio`,
              iconType,
              pid: app.pid,
              nodeType: "app",
            });
            setActiveModal(null);
          }}
          onSelectInputDevice={(dev) => {
            onAddSource({
              name: dev.name,
              subtext: `Hardware Mic • ${dev.sample_rate} Hz`,
              iconType: "mic",
              nodeType: "mic",
            });
            setActiveModal(null);
          }}
          onClose={() => setActiveModal(null)}
        />
      )}

      {/* Modal: Create Virtual Device */}
      {activeModal === "virtual" && (
        <CreateVirtualModal
          onCreate={(name: string, driverType: string, channels: number) => {
            onAddVirtualDevice({
              name,
              subtext: `Virtual Cable (${channels} Ch, ${driverType})`,
              driverType,
              channels,
            });
            setActiveModal(null);
          }}
          onClose={() => setActiveModal(null)}
        />
      )}

      {/* Modal: Add Output */}
      {activeModal === "output" && (
        <AddOutputModal
          devices={devices}
          onSelectOutput={(dev, iconType) => {
            onAddOutput({
              name: dev.name,
              subtext: `Output • ${dev.sample_rate} Hz`,
              iconType,
            });
            setActiveModal(null);
          }}
          onClose={() => setActiveModal(null)}
        />
      )}
    </>
  );
}

export const TopBar = forwardRef<TopBarHandle, TopBarProps>(TopBarRender);
export default TopBar;
