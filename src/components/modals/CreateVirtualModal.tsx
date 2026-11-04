/**
 * @file CreateVirtualModal.tsx
 * @brief Modal dialog facilitating the creation, configuration, and driver lifecycle of virtual audio cables.
 * @details Supports multi-channel virtual device allocation (2, 4, 8, 16 channels) and on-demand Windows
 * driver installation/uninstallation with Administrator elevation.
 */

import React, { useState, useEffect } from "react";
import {
  Layers,
  X,
  AlertCircle,
  CheckCircle2,
  Download,
  Trash2,
  Loader2,
  ShieldAlert,
} from "lucide-react";
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
  const [virtualDriver, setVirtualDriver] = useState("VBCable");
  const [channelCount, setChannelCount] = useState<number>(2);
  const [isDriverInstalled, setIsDriverInstalled] = useState<boolean>(false);
  const [isLoadingDriver, setIsLoadingDriver] = useState<boolean>(true);
  const [isBusy, setIsBusy] = useState<boolean>(false);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  /**
   * @brief Queries the Windows Virtual Cable driver status on mount.
   */
  const checkStatus = async () => {
    setIsLoadingDriver(true);
    try {
      const installed = await invoke<boolean>("check_cable_driver_status");
      setIsDriverInstalled(Boolean(installed));
    } catch {
      setIsDriverInstalled(false);
    } finally {
      setIsLoadingDriver(false);
    }
  };

  useEffect(() => {
    checkStatus();
  }, []);

  /**
   * @brief Handles on-demand driver installation with Administrator elevation.
   */
  const handleInstallDriver = async () => {
    setIsBusy(true);
    setErrorMessage(null);
    setStatusMessage("Đang chuẩn bị gói driver & yêu cầu quyền Administrator (UAC)... Vui lòng xác nhận trên màn hình.");
    try {
      const result = await invoke<string>("install_cable_driver");
      setStatusMessage(result);
      await checkStatus();
    } catch (err) {
      setErrorMessage(typeof err === "string" ? err : "Cài đặt driver thất bại hoặc bị huỷ bởi người dùng.");
      setStatusMessage(null);
    } finally {
      setIsBusy(false);
    }
  };

  /**
   * @brief Handles complete driver uninstallation from Windows.
   */
  const handleUninstallDriver = async () => {
    const confirmed = window.confirm(
      "Bạn có chắc chắn muốn gỡ bỏ hoàn toàn driver Windows Virtual Cable khỏi hệ thống Windows? (CABLE Input & CABLE Output sẽ bị xoá khỏi Windows Sound Settings)"
    );
    if (!confirmed) return;

    setIsBusy(true);
    setErrorMessage(null);
    setStatusMessage("Đang yêu cầu quyền Administrator để gỡ bỏ driver khỏi Windows... Vui lòng xác nhận trên màn hình.");
    try {
      const result = await invoke<string>("uninstall_cable_driver");
      setStatusMessage(result);
      await checkStatus();
    } catch (err) {
      setErrorMessage(typeof err === "string" ? err : "Gỡ cài đặt driver thất bại hoặc bị huỷ bởi người dùng.");
      setStatusMessage(null);
    } finally {
      setIsBusy(false);
    }
  };

  /**
   * @brief Handles form submission and backend registration.
   * @param e Form event.
   */
  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setErrorMessage(null);

    const trimmed = virtualName.trim() || (virtualDriver === "VBCable" ? "Windows Virtual Cable" : "Virtual Audio Bus");

    if (virtualDriver === "VBCable" && !isDriverInstalled) {
      setErrorMessage("Vui lòng bấm 'Cài đặt Driver trên Windows' trước khi tạo thiết bị Windows Virtual Cable, hoặc chọn 'Dyno Software Loopback'.");
      return;
    }

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
      <div className="w-full max-w-lg rounded-2xl bg-slate-900 border border-slate-700 shadow-2xl overflow-hidden">
        <div className="flex items-center justify-between px-5 py-3.5 border-b border-slate-800">
          <div className="flex items-center gap-2">
            <Layers className="w-4 h-4 text-violet-400" />
            <h2 className="text-sm font-semibold text-white">Create Virtual Audio Device (Thiết bị ảo)</h2>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="p-1 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 transition-colors"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        <form onSubmit={handleSubmit} className="p-5 flex flex-col gap-4">
          {/* Windows Driver Status Banner */}
          <div className="rounded-xl border p-3.5 text-xs transition-all bg-slate-950/60 border-slate-800">
            <div className="flex items-start justify-between gap-3">
              <div className="flex items-start gap-2.5">
                {isLoadingDriver ? (
                  <Loader2 className="w-4 h-4 text-slate-400 animate-spin shrink-0 mt-0.5" />
                ) : isDriverInstalled ? (
                  <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0 mt-0.5" />
                ) : (
                  <AlertCircle className="w-4 h-4 text-amber-400 shrink-0 mt-0.5" />
                )}
                <div>
                  <h4 className="font-semibold text-slate-200">
                    {isLoadingDriver
                      ? "Đang kiểm tra driver Windows..."
                      : isDriverInstalled
                      ? "Windows Virtual Cable: Đã kích hoạt"
                      : "Windows Virtual Cable: Chưa cài đặt"}
                  </h4>
                  <p className="text-[11px] text-slate-400 mt-1 leading-relaxed">
                    {isDriverInstalled
                      ? "Thiết bị 'CABLE Input' và 'CABLE Output' đang hoạt động trên Windows Sound Settings (mmsys.cpl) và mọi phần mềm khác (Discord, OBS, Game)."
                      : "Để thiết bị xuất hiện trong Windows Sound Settings và các app khác, cần cài đặt driver Windows Virtual Cable (WHQL-signed). Dyno không tự ý cài đặt ngầm driver nào khi cài app."}
                  </p>
                </div>
              </div>

              {/* Action button inside banner */}
              {!isLoadingDriver && (
                <div className="shrink-0">
                  {isDriverInstalled ? (
                    <button
                      type="button"
                      onClick={handleUninstallDriver}
                      disabled={isBusy}
                      title="Gỡ bỏ hoàn toàn driver Virtual Cable khỏi hệ thống Windows"
                      className="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg text-[11px] font-medium bg-rose-500/15 border border-rose-500/30 text-rose-300 hover:bg-rose-500/25 transition-all cursor-pointer disabled:opacity-50"
                    >
                      {isBusy ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <Trash2 className="w-3.5 h-3.5" />}
                      <span>Gỡ khỏi Windows</span>
                    </button>
                  ) : (
                    <button
                      type="button"
                      onClick={handleInstallDriver}
                      disabled={isBusy}
                      title="Cài đặt driver Windows Virtual Cable (Yêu cầu quyền Admin UAC)"
                      className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-[11px] font-medium bg-emerald-500/20 border border-emerald-500/40 text-emerald-300 hover:bg-emerald-500/30 shadow-sm transition-all cursor-pointer disabled:opacity-50"
                    >
                      {isBusy ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <Download className="w-3.5 h-3.5" />}
                      <span>Cài đặt Driver</span>
                    </button>
                  )}
                </div>
              )}
            </div>

            {/* Status Feedback Messages */}
            {statusMessage && (
              <div className="mt-2.5 pt-2 border-t border-slate-800 text-[11px] text-emerald-400 flex items-center gap-1.5">
                <CheckCircle2 className="w-3.5 h-3.5 shrink-0" />
                <span>{statusMessage}</span>
              </div>
            )}
            {errorMessage && (
              <div className="mt-2.5 pt-2 border-t border-slate-800 text-[11px] text-rose-400 flex items-center gap-1.5">
                <ShieldAlert className="w-3.5 h-3.5 shrink-0" />
                <span>{errorMessage}</span>
              </div>
            )}
          </div>

          <div>
            <label className="block text-xs font-medium text-slate-300 mb-1.5">
              Device Name (Tên thiết bị)
            </label>
            <input
              type="text"
              placeholder={virtualDriver === "VBCable" ? "VB-Audio Cable (CABLE Input/Output)" : "e.g. Stream Mix, Discord Aux, OBS Pass"}
              value={virtualName}
              onChange={(e) => setVirtualName(e.target.value)}
              className="w-full px-3 py-2 rounded-lg bg-slate-950/80 border border-slate-700 text-xs text-white placeholder-slate-500 focus:outline-none focus:border-violet-500"
              autoFocus
            />
          </div>

          <div>
            <label className="block text-xs font-medium text-slate-300 mb-1.5">
              Driver Technology (Loại thiết bị)
            </label>
            <select
              value={virtualDriver}
              onChange={(e) => setVirtualDriver(e.target.value)}
              className="w-full px-3 py-2 rounded-lg bg-slate-950/80 border border-slate-700 text-xs text-white focus:outline-none focus:border-violet-500"
            >
              <option value="VBCable">
                Windows System Virtual Cable (Xuất hiện trong Windows Sound Settings &amp; các app khác)
              </option>
              <option value="SoftwareLoopbackEndpoint">
                Dyno Internal Software Bus (Zero-driver, chỉ hoạt động nội bộ trong mixer)
              </option>
            </select>
          </div>

          <div>
            <label className="block text-xs font-medium text-slate-300 mb-1.5">
              Channel Configuration (Số kênh âm thanh)
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

          <div className="pt-2 flex items-center justify-between">
            <span className="text-[10px] text-slate-500">
              User có toàn quyền quản lý &amp; xoá bỏ thiết bị khỏi Windows bất kỳ lúc nào.
            </span>
            <div className="flex items-center gap-2.5">
              <button
                type="button"
                onClick={onClose}
                className="px-3.5 py-1.5 rounded-lg text-xs font-medium text-slate-400 hover:text-white hover:bg-slate-800 transition-colors"
              >
                Hủy (Cancel)
              </button>
              <button
                type="submit"
                disabled={isBusy || (virtualDriver === "VBCable" && !isDriverInstalled)}
                className="flex items-center gap-1.5 px-4 py-1.5 rounded-lg text-xs font-medium bg-violet-600 hover:bg-violet-500 text-white shadow-md transition-all cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed"
              >
                <Layers className="w-3.5 h-3.5" />
                <span>Tạo &amp; Thêm Thiết Bị</span>
              </button>
            </div>
          </div>
        </form>
      </div>
    </div>
  );
}

export default CreateVirtualModal;
