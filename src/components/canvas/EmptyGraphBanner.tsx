/**
 * @file EmptyGraphBanner.tsx
 * @brief Guidance overlay banner shown when no audio nodes are present in the patchbay matrix.
 * @details Guides users to either auto-populate their real detected hardware devices or add apps/outputs manually.
 */

import React from "react";
import { Radio, Zap, Plus, Volume2 } from "lucide-react";

/**
 * @brief Props definition for EmptyGraphBanner component.
 */
export interface EmptyGraphBannerProps {
  /** @brief Callback to automatically populate real system hardware endpoints. */
  onResetHardware: () => void;
  /** @brief Callback to open the Add Source modal. */
  onOpenAddSource?: () => void;
  /** @brief Callback to open the Add Output modal. */
  onOpenAddOutput?: () => void;
}

/**
 * @brief Empty state canvas guidance overlay banner.
 * @param {EmptyGraphBannerProps} props Component properties.
 * @return {React.JSX.Element} Rendered empty state guidance banner.
 */
export function EmptyGraphBanner({
  onResetHardware,
  onOpenAddSource,
  onOpenAddOutput,
}: EmptyGraphBannerProps): React.JSX.Element {
  return (
    <div className="absolute inset-0 pointer-events-none flex items-center justify-center p-6">
      <div className="pointer-events-auto max-w-md w-full rounded-2xl bg-slate-900/90 border border-slate-800 shadow-2xl p-6 text-center backdrop-blur-md flex flex-col items-center gap-4">
        <div className="w-12 h-12 rounded-xl bg-sky-500/10 border border-sky-500/20 text-sky-400 flex items-center justify-center">
          <Radio className="w-6 h-6 animate-pulse" />
        </div>
        <div>
          <h3 className="text-base font-semibold text-white">Canvas Trống</h3>
          <p className="text-xs text-slate-400 mt-1 leading-relaxed">
            Chưa có luồng âm thanh nào được thêm vào ma trận. Bạn có thể tự động nạp thiết bị mặc định của máy hoặc thêm ứng dụng thủ công.
          </p>
        </div>
        <div className="flex flex-col w-full gap-2 pt-2">
          <button
            type="button"
            onClick={onResetHardware}
            className="flex items-center justify-center gap-2 px-4 py-2.5 rounded-xl bg-sky-500 hover:bg-sky-400 text-slate-950 text-xs font-semibold shadow-lg shadow-sky-500/20 transition-all cursor-pointer"
          >
            <Zap className="w-4 h-4 fill-current" />
            Tự Động Nạp Micro &amp; Loa Thật Của Máy
          </button>
          <div className="flex gap-2">
            <button
              type="button"
              onClick={() => onOpenAddSource?.()}
              className="flex-1 flex items-center justify-center gap-1.5 px-3 py-2 rounded-xl bg-slate-800 hover:bg-slate-700 border border-slate-700/60 text-slate-200 text-xs font-medium transition-all cursor-pointer"
            >
              <Plus className="w-3.5 h-3.5 text-sky-400" />
              Thêm Nguồn
            </button>
            <button
              type="button"
              onClick={() => onOpenAddOutput?.()}
              className="flex-1 flex items-center justify-center gap-1.5 px-3 py-2 rounded-xl bg-slate-800 hover:bg-slate-700 border border-slate-700/60 text-slate-200 text-xs font-medium transition-all cursor-pointer"
            >
              <Volume2 className="w-3.5 h-3.5 text-emerald-400" />
              Thêm Đầu Ra
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

export default EmptyGraphBanner;
