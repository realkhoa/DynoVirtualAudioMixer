/**
 * @file AddSourceModal.tsx
 * @brief Modal dialog allowing the user to select and add active applications or microphone inputs.
 */

import React, { useState } from "react";
import { Radio, X, Music, Globe, Gamepad2, Mic } from "lucide-react";
import type {
  AudioProcessDescriptor,
  AudioDeviceDescriptor,
  AudioIconType,
} from "../../types/audio";

/**
 * @brief Props definition for AddSourceModal.
 */
export interface AddSourceModalProps {
  /** @brief Available scanned applications. */
  apps: AudioProcessDescriptor[];
  /** @brief Available hardware audio devices. */
  devices: AudioDeviceDescriptor[];
  /** @brief Callback when an application is selected. */
  onSelectApp: (app: AudioProcessDescriptor, iconType: AudioIconType) => void;
  /** @brief Callback when an input device is selected. */
  onSelectInputDevice: (dev: AudioDeviceDescriptor) => void;
  /** @brief Callback to close the modal. */
  onClose: () => void;
}

/**
 * @brief Derives visual icon type from process executable name.
 * @param exeName Binary file name.
 * @return {AudioIconType} Categorized icon name.
 */
export function resolveAppIcon(exeName: string): AudioIconType {
  const lower = exeName.toLowerCase();
  if (lower.includes("spotify") || lower.includes("music") || lower.includes("tidal")) {
    return "music";
  }
  if (lower.includes("chrome") || lower.includes("edge") || lower.includes("firefox")) {
    return "browser";
  }
  if (lower.includes("game") || lower.includes("steam") || lower.includes("cyberpunk")) {
    return "game";
  }
  return "app";
}

/**
 * @brief Add Audio Source modal dialog.
 * @param {AddSourceModalProps} props Component properties.
 * @return {React.JSX.Element} Rendered modal dialog.
 */
export function AddSourceModal({
  apps,
  devices,
  onSelectApp,
  onSelectInputDevice,
  onClose,
}: AddSourceModalProps): React.JSX.Element {
  const [sourceTab, setSourceTab] = useState<"apps" | "mics">("apps");
  const inputDevices = devices.filter((d) => d.device_type === "Input");

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4">
      <div className="w-full max-w-lg rounded-2xl bg-slate-900 border border-slate-700 shadow-2xl overflow-hidden flex flex-col max-h-[80vh]">
        <div className="flex items-center justify-between px-5 py-3.5 border-b border-slate-800">
          <div className="flex items-center gap-2">
            <Radio className="w-4 h-4 text-sky-400" />
            <h2 className="text-sm font-semibold text-white">Add Audio Source</h2>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="p-1 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Tab switch */}
        <div className="flex border-b border-slate-800 bg-slate-950/40 px-5 pt-2 gap-4">
          <button
            type="button"
            onClick={() => setSourceTab("apps")}
            className={`pb-2.5 text-xs font-medium border-b-2 transition-all ${
              sourceTab === "apps"
                ? "border-sky-400 text-sky-300"
                : "border-transparent text-slate-400 hover:text-slate-200"
            }`}
          >
            Applications ({apps.length})
          </button>
          <button
            type="button"
            onClick={() => setSourceTab("mics")}
            className={`pb-2.5 text-xs font-medium border-b-2 transition-all ${
              sourceTab === "mics"
                ? "border-sky-400 text-sky-300"
                : "border-transparent text-slate-400 hover:text-slate-200"
            }`}
          >
            Microphones &amp; Inputs ({inputDevices.length})
          </button>
        </div>

        {/* List */}
        <div className="p-4 overflow-y-auto flex-1 flex flex-col gap-2">
          {sourceTab === "apps" &&
            apps.map((app) => (
              <button
                key={`${app.pid}-${app.name}`}
                type="button"
                onClick={() => onSelectApp(app, resolveAppIcon(app.name))}
                className="flex items-center justify-between p-3 rounded-xl bg-slate-800/50 hover:bg-slate-800 border border-slate-700/60 hover:border-sky-500/50 text-left transition-all group"
              >
                <div className="flex items-center gap-3 min-w-0">
                  <div className="flex items-center justify-center w-8 h-8 rounded-lg bg-slate-700/60 border border-slate-600/50 text-sky-400">
                    {app.name.toLowerCase().includes("spotify") ? (
                      <Music className="w-4 h-4" />
                    ) : app.name.toLowerCase().includes("chrome") ? (
                      <Globe className="w-4 h-4" />
                    ) : app.name.toLowerCase().includes("game") ? (
                      <Gamepad2 className="w-4 h-4" />
                    ) : (
                      <Radio className="w-4 h-4" />
                    )}
                  </div>
                  <div className="min-w-0">
                    <p className="text-xs font-semibold text-slate-200 group-hover:text-sky-300 truncate">
                      {app.title || app.name}
                    </p>
                    <p className="text-[10px] text-slate-400 truncate">
                      Process: {app.name} • PID: {app.pid}
                    </p>
                  </div>
                </div>
                <span className="px-2 py-1 rounded text-[10px] font-medium bg-sky-500/10 text-sky-400 border border-sky-500/30 group-hover:bg-sky-500 group-hover:text-slate-950 transition-all shrink-0">
                  + Add
                </span>
              </button>
            ))}

          {sourceTab === "mics" &&
            inputDevices.map((dev) => (
              <button
                key={dev.id}
                type="button"
                onClick={() => onSelectInputDevice(dev)}
                className="flex items-center justify-between p-3 rounded-xl bg-slate-800/50 hover:bg-slate-800 border border-slate-700/60 hover:border-emerald-500/50 text-left transition-all group"
              >
                <div className="flex items-center gap-3 min-w-0">
                  <div className="flex items-center justify-center w-8 h-8 rounded-lg bg-emerald-500/10 border border-emerald-500/20 text-emerald-400">
                    <Mic className="w-4 h-4" />
                  </div>
                  <div className="min-w-0">
                    <p className="text-xs font-semibold text-slate-200 group-hover:text-emerald-300 truncate">
                      {dev.name}
                    </p>
                    <p className="text-[10px] text-slate-400">
                      {dev.channels} Channels • {dev.sample_rate} Hz
                      {dev.is_default && " • Default Input"}
                    </p>
                  </div>
                </div>
                <span className="px-2 py-1 rounded text-[10px] font-medium bg-emerald-500/10 text-emerald-400 border border-emerald-500/30 group-hover:bg-emerald-500 group-hover:text-slate-950 transition-all shrink-0">
                  + Add
                </span>
              </button>
            ))}
        </div>
      </div>
    </div>
  );
}

export default AddSourceModal;
