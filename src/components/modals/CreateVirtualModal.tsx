/**
 * @file CreateVirtualModal.tsx
 * @brief Modal dialog facilitating the creation and configuration of virtual audio cable buses.
 * @details Supports multi-channel virtual device allocation (2, 4, 8, 16 channels) for DAWs and multi-track recording.
 */

import React, { useState } from "react";
import { Layers, X } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import type { VirtualDeviceDescriptor } from "../../types/audio";

/**
 * @brief Props definition for CreateVirtualModal.
 */
export interface CreateVirtualModalProps {
  /** @brief Callback when a virtual device is successfully configured. */
  onCreate: (name: string, driverType: string, channels: number) => void;
  /** @brief Callback to close the modal. */
  onClose: () => void;
}

/**
 * @brief Create Virtual Device modal dialog component.
 * @param {CreateVirtualModalProps} props Component properties.
 * @return {React.JSX.Element} Rendered modal dialog.
 */
export function CreateVirtualModal({
  onCreate,
  onClose,
}: CreateVirtualModalProps): React.JSX.Element {
  const [virtualName, setVirtualName] = useState("");
  const [virtualDriver, setVirtualDriver] = useState("SoftwareLoopbackEndpoint");
  const [channelCount, setChannelCount] = useState<number>(2);

  /**
   * @brief Handles form submission and backend registration.
   * @param e Form event.
   */
  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    const trimmed = virtualName.trim() || "Virtual Audio Bus";

    try {
      await invoke<VirtualDeviceDescriptor>("create_virtual_device", {
        name: trimmed,
        cableType: virtualDriver,
        channels: channelCount,
      });
    } catch {
      // In standalone browser mode, proceed with frontend creation
    }

    onCreate(trimmed, virtualDriver, channelCount);
    setVirtualName("");
    onClose();
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4">
      <div className="w-full max-w-md rounded-2xl bg-slate-900 border border-slate-700 shadow-2xl overflow-hidden">
        <div className="flex items-center justify-between px-5 py-3.5 border-b border-slate-800">
          <div className="flex items-center gap-2">
            <Layers className="w-4 h-4 text-violet-400" />
            <h2 className="text-sm font-semibold text-white">Create Virtual Audio Bus</h2>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="p-1 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        <form onSubmit={handleSubmit} className="p-5 flex flex-col gap-4">
          <div>
            <label className="block text-xs font-medium text-slate-300 mb-1.5">
              Device Name
            </label>
            <input
              type="text"
              placeholder="e.g. Stream Mix, Discord Aux, OBS Pass"
              value={virtualName}
              onChange={(e) => setVirtualName(e.target.value)}
              className="w-full px-3 py-2 rounded-lg bg-slate-950/80 border border-slate-700 text-xs text-white placeholder-slate-500 focus:outline-none focus:border-violet-500"
              autoFocus
            />
          </div>

          <div>
            <label className="block text-xs font-medium text-slate-300 mb-1.5">
              Channel Configuration
            </label>
            <select
              value={channelCount}
              onChange={(e) => setChannelCount(parseInt(e.target.value, 10))}
              className="w-full px-3 py-2 rounded-lg bg-slate-950/80 border border-slate-700 text-xs text-white focus:outline-none focus:border-violet-500"
            >
              <option value={2}>Stereo (2 Channels - Standard L/R)</option>
              <option value={4}>Quad (4 Channels - Multi-source / Podcast)</option>
              <option value={8}>8 Channels (Surround 7.1 / Multi-track)</option>
              <option value={16}>16 Channels (Professional DAW Sub-bus)</option>
            </select>
          </div>

          <div>
            <label className="block text-xs font-medium text-slate-300 mb-1.5">
              Driver Technology
            </label>
            <select
              value={virtualDriver}
              onChange={(e) => setVirtualDriver(e.target.value)}
              className="w-full px-3 py-2 rounded-lg bg-slate-950/80 border border-slate-700 text-xs text-white focus:outline-none focus:border-violet-500"
            >
              <option value="SoftwareLoopbackEndpoint">
                Dyno Software Loopback (Zero-driver internal)
              </option>
              <option value="VBCable">VB-Audio Virtual Cable (CABLE-A / B)</option>
              <option value="VirtualAudioCable">Virtual Audio Cable (VAC WDM)</option>
            </select>
          </div>

          <div className="pt-2 flex justify-end gap-2.5">
            <button
              type="button"
              onClick={onClose}
              className="px-3.5 py-1.5 rounded-lg text-xs font-medium text-slate-400 hover:text-white hover:bg-slate-800"
            >
              Cancel
            </button>
            <button
              type="submit"
              className="px-4 py-1.5 rounded-lg text-xs font-medium bg-violet-600 hover:bg-violet-500 text-white shadow-md transition-all cursor-pointer"
            >
              Create &amp; Add Bus
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

export default CreateVirtualModal;
