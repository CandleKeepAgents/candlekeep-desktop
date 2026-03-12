import { Info, LogOut, RefreshCw } from "lucide-react";
import { useState } from "react";
import { authLogout, checkAppUpdate } from "../lib/tauri-commands";
import type { AppUpdateInfo } from "../lib/types";

export function Settings() {
  const [updateInfo, setUpdateInfo] = useState<AppUpdateInfo | null>(null);
  const [checkingUpdate, setCheckingUpdate] = useState(false);

  const handleCheckUpdate = async () => {
    setCheckingUpdate(true);
    try {
      const info = await checkAppUpdate();
      setUpdateInfo(info);
    } catch (err) {
      console.error("Failed to check for updates:", err);
    } finally {
      setCheckingUpdate(false);
    }
  };

  const handleLogout = async () => {
    try {
      await authLogout();
    } catch (err) {
      console.error("Failed to logout:", err);
    }
  };

  return (
    <div className="space-y-4">
      <h2 className="text-lg font-semibold text-zinc-100">Settings</h2>

      <div className="space-y-3">
        <div className="p-4 rounded-lg bg-zinc-800/50 border border-zinc-700/50 space-y-3">
          <h3 className="text-sm font-medium text-zinc-300 flex items-center gap-2">
            <Info className="w-4 h-4" />
            About
          </h3>
          <div className="text-xs text-zinc-400 space-y-1">
            <p>CandleKeep Desktop v0.1.0</p>
            <p>Give AI agents direct access to your books.</p>
          </div>
        </div>

        <div className="p-4 rounded-lg bg-zinc-800/50 border border-zinc-700/50 space-y-3">
          <h3 className="text-sm font-medium text-zinc-300 flex items-center gap-2">
            <RefreshCw className="w-4 h-4" />
            Updates
          </h3>
          <button
            type="button"
            onClick={handleCheckUpdate}
            disabled={checkingUpdate}
            className="w-full text-xs px-3 py-2 rounded-md bg-zinc-700 hover:bg-zinc-600 disabled:opacity-50 text-zinc-300 transition-colors"
          >
            {checkingUpdate ? "Checking..." : "Check for Updates"}
          </button>
          {updateInfo && (
            <p className="text-xs text-zinc-400">
              {updateInfo.update_available
                ? `Update available: v${updateInfo.latest_version}`
                : `You're on the latest version (v${updateInfo.current_version})`}
            </p>
          )}
        </div>

        <div className="p-4 rounded-lg bg-zinc-800/50 border border-zinc-700/50 space-y-3">
          <h3 className="text-sm font-medium text-zinc-300 flex items-center gap-2">
            <LogOut className="w-4 h-4" />
            Account
          </h3>
          <button
            type="button"
            onClick={handleLogout}
            className="w-full text-xs px-3 py-2 rounded-md bg-red-900/30 hover:bg-red-900/50 text-red-400 border border-red-800/50 transition-colors"
          >
            Sign Out
          </button>
        </div>
      </div>
    </div>
  );
}
