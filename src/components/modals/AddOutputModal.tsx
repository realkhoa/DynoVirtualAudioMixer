/**
 * @file AddOutputModal.tsx
 * @brief Modal dialog facilitating selection and injection of physical or virtual audio output endpoints.
 */

import React from "react";
import { Volume2, X, Headphones, Speaker } from "lucide-react";
import type { AudioDeviceDescriptor, AudioIconType } from "../../types/audio";

/**
 * @brief Props definition for AddOutputModal.
 */
export interface AddOutputModalProps {
  /** @brief Available hardware audio devices. */
  devices: AudioDeviceDescriptor[];
  /** @brief Callback when an output destination is selected. */
  onSelectOutput: (dev: AudioDeviceDescriptor, iconType: AudioIconType) => void;
  /** @brief Callback to close the modal. */
  onClose: () => void;
}

/**
 * @brief Add Output Destination modal dialog component.
 * @param {AddOutputModalProps} props Component properties.
 * @return {React.JSX.Element} Rendered modal dialog.
 */
export function AddOutputModal({
  devices,
  onSelectOutput,
  onClose,
}: AddOutputModalProps): React.JSX.Element {
  const outputDevices = devices.filter((d) => d.device_type === "Output");

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4">
      <div className="w-full max-w-lg rounded-2xl bg-slate-900 border border-slate-700 shadow-2xl overflow-hidden flex flex-col max-h-[80vh]">
        <div className="flex items-center justify-between px-5 py-3.5 border-b border-slate-800">
          <div className="flex items-center gap-2">
            <Volume2 className="w-4 h-4 text-emerald-400" />
            <h2 className="text-sm font-semibold text-white">Add Output Destination</h2>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="p-1 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        <div className="p-4 overflow-y-auto flex-1 flex flex-col gap-2">
          {outputDevices.map((dev) => {
            const isPhones = dev.name.toLowerCase().includes("headphone");
            const iconType: AudioIconType = isPhones ? "headphones" : "speaker";

            return (
              <button
                key={dev.id}
                type="button"
                onClick={() => onSelectOutput(dev, iconType)}
                className="flex items-center justify-between p-3 rounded-xl bg-slate-800/50 hover:bg-slate-800 border border-slate-700/60 hover:border-emerald-500/50 text-left transition-all group"
              >
                <div className="flex items-center gap-3 min-w-0">
                  <div className="flex items-center justify-center w-8 h-8 rounded-lg bg-emerald-500/10 border border-emerald-500/20 text-emerald-400">
                    {isPhones ? (
                      <Headphones className="w-4 h-4" />
                    ) : (
                      <Speaker className="w-4 h-4" />
                    )}
                  </div>
                  <div className="min-w-0">
                    <p className="text-xs font-semibold text-slate-200 group-hover:text-emerald-300 truncate">
                      {dev.name}
                    </p>
                    <p className="text-[10px] text-slate-400">
                      {dev.channels} Channels • {dev.sample_rate} Hz
                      {dev.is_default && " • Default Output"}
                    </p>
                  </div>
                </div>
                <span className="px-2 py-1 rounded text-[10px] font-medium bg-emerald-500/10 text-emerald-400 border border-emerald-500/30 group-hover:bg-emerald-500 group-hover:text-slate-950 transition-all shrink-0">
                  + Add
                </span>
              </button>
            );
          })}
        </div>
      </div>
    </div>
  );
}

export default AddOutputModal;
