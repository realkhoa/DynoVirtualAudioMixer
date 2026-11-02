/**
 * @file TopBar.tsx
 * @brief Application header bar and modal managers for adding sources, virtual buses, and outputs.
 * @details Queries active WASAPI devices, running applications, and virtual cables via Tauri IPC,
 * rendering responsive modal dialogs to inject new nodes into the React Flow patchbay graph.
 */

import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Radio, Plus, Cpu, Activity } from "lucide-react";
import type {
  AudioDeviceDescriptor,
  AudioProcessDescriptor,
  AudioIconType,
} from "../types/audio";
import { AddSourceModal } from "./modals/AddSourceModal";
import { CreateVirtualModal } from "./modals/CreateVirtualModal";
import { AddOutputModal } from "./modals/AddOutputModal";

/**
 * @brief Fallback mock application sessions used when running outside the Tauri runtime.
 */
const FALLBACK_APPS: AudioProcessDescriptor[] = [
  { pid: 1420, name: "spotify.exe", title: "Spotify Free", has_audio_session: true },
  { pid: 3824, name: "discord.exe", title: "Discord - Voice Channel", has_audio_session: true },
  { pid: 8192, name: "chrome.exe", title: "YouTube - Chrome", has_audio_session: true },
  { pid: 5120, name: "game.exe", title: "Cyberpunk 2077", has_audio_session: true },
];

/**
 * @brief Fallback mock audio devices used when running outside the Tauri runtime.
 */
const FALLBACK_DEVICES: AudioDeviceDescriptor[] = [
  {
    id: "hw-in-mic-1",
    name: "Microphone (Realtek High Definition Audio)",
    device_type: "Input",
    channels: 2,
    sample_rate: 48000,
    is_default: true,
  },
  {
    id: "hw-in-shure-2",
    name: "Shure MV7+ USB Podcasting Mic",
    device_type: "Input",
    channels: 1,
    sample_rate: 48000,
    is_default: false,
  },
  {
    id: "hw-out-phones-1",
    name: "Headphones (Realtek High Definition Audio)",
    device_type: "Output",
    channels: 2,
    sample_rate: 48000,
    is_default: true,
  },
  {
    id: "hw-out-speakers-2",
    name: "Studio Monitors (Focusrite Scarlett 2i2)",
    device_type: "Output",
    channels: 2,
    sample_rate: 48000,
    is_default: false,
  },
];

/**
 * @brief Props definition for the TopBar component.
 */
export interface TopBarProps {
  /** @brief Callback to inject a new source node into the patchbay. */
  onAddSource: (payload: { name: string; subtext: string; iconType: AudioIconType }) => void;
  /** @brief Callback to inject a new virtual device node into the patchbay. */
  onAddVirtualDevice: (payload: { name: string; subtext: string; driverType?: string }) => void;
  /** @brief Callback to inject a new output node into the patchbay. */
  onAddOutput: (payload: { name: string; subtext: string; iconType: AudioIconType }) => void;
}

/**
 * @brief Top navigation bar and modal manager for patchbay graph operations.
 * @param {TopBarProps} props Callback functions for graph mutations.
 * @return {React.JSX.Element} Rendered TopBar header.
 */
export function TopBar({
  onAddSource,
  onAddVirtualDevice,
  onAddOutput,
}: TopBarProps): React.JSX.Element {
  const [activeModal, setActiveModal] = useState<"source" | "virtual" | "output" | null>(null);
  const [apps, setApps] = useState<AudioProcessDescriptor[]>([]);
  const [devices, setDevices] = useState<AudioDeviceDescriptor[]>([]);

  useEffect(() => {
    let isMounted = true;

    async function loadData() {
      try {
        const runningApps = await invoke<AudioProcessDescriptor[]>("get_running_apps");
        if (isMounted) setApps(runningApps.length > 0 ? runningApps : FALLBACK_APPS);
      } catch {
        if (isMounted) setApps(FALLBACK_APPS);
      }

      try {
        const devList = await invoke<AudioDeviceDescriptor[]>("get_devices");
        if (isMounted) setDevices(devList.length > 0 ? devList : FALLBACK_DEVICES);
      } catch {
        if (isMounted) setDevices(FALLBACK_DEVICES);
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
        <div className="flex items-center gap-2.5">
          <button
            type="button"
            onClick={() => setActiveModal("source")}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium bg-sky-500/15 border border-sky-500/30 text-sky-300 hover:bg-sky-500/25 hover:border-sky-500/50 shadow-sm transition-all"
          >
            <Plus className="w-3.5 h-3.5" />
            <span>Add Source</span>
          </button>

          <button
            type="button"
            onClick={() => setActiveModal("virtual")}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium bg-violet-500/15 border border-violet-500/30 text-violet-300 hover:bg-violet-500/25 hover:border-violet-500/50 shadow-sm transition-all"
          >
            <Plus className="w-3.5 h-3.5" />
            <span>New Virtual Device</span>
          </button>

          <button
            type="button"
            onClick={() => setActiveModal("output")}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium bg-emerald-500/15 border border-emerald-500/30 text-emerald-300 hover:bg-emerald-500/25 hover:border-emerald-500/50 shadow-sm transition-all"
          >
            <Plus className="w-3.5 h-3.5" />
            <span>Add Output</span>
          </button>
        </div>

        {/* Telemetry Status Badges */}
        <div className="flex items-center gap-2 text-xs">
          <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-md bg-slate-800/80 border border-slate-700/60">
            <Cpu className="w-3.5 h-3.5 text-sky-400" />
            <span className="text-slate-400 text-[11px]">48,000 Hz</span>
          </div>

          <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-md bg-slate-800/80 border border-slate-700/60">
            <Activity className="w-3.5 h-3.5 text-emerald-400" />
            <span className="text-emerald-400 font-mono text-[11px]">&lt; 10 ms</span>
          </div>

          <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-md bg-slate-800/80 border border-slate-700/60">
            <span className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse" />
            <span className="text-emerald-400 text-[11px] font-medium">Engine Active</span>
          </div>
        </div>
      </header>

      {/* Render active modal */}
      {activeModal === "source" && (
        <AddSourceModal
          apps={apps}
          devices={devices}
          onSelectApp={(app, icon) => {
            onAddSource({
              name: app.title || app.name,
              subtext: `PID: ${app.pid} • ${app.name}`,
              iconType: icon,
            });
            setActiveModal(null);
          }}
          onSelectInputDevice={(dev) => {
            onAddSource({
              name: dev.name,
              subtext: `${dev.channels}ch • ${dev.sample_rate / 1000} kHz`,
              iconType: "mic",
            });
            setActiveModal(null);
          }}
          onClose={() => setActiveModal(null)}
        />
      )}

      {activeModal === "virtual" && (
        <CreateVirtualModal
          onCreate={(name, driver) => {
            onAddVirtualDevice({
              name,
              subtext:
                driver === "VBCable"
                  ? "VB-Audio Cable Driver"
                  : driver === "VirtualAudioCable"
                  ? "VAC Driver"
                  : "Dyno Software Loopback",
              driverType: driver,
            });
          }}
          onClose={() => setActiveModal(null)}
        />
      )}

      {activeModal === "output" && (
        <AddOutputModal
          devices={devices}
          onSelectOutput={(dev, icon) => {
            onAddOutput({
              name: dev.name,
              subtext: `${dev.channels}ch • ${dev.sample_rate / 1000} kHz`,
              iconType: icon,
            });
            setActiveModal(null);
          }}
          onClose={() => setActiveModal(null)}
        />
      )}
    </>
  );
}

export default TopBar;
