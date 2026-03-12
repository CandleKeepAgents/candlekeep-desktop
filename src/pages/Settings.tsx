import { Info, LogOut, RefreshCw } from "lucide-react";
import { useState } from "react";
import { authLogout, checkAppUpdate } from "../lib/tauri-commands";
import type { AppUpdateInfo } from "../lib/types";

export function Settings({ onLogout }: { onLogout: () => void }) {
  const [updateInfo, setUpdateInfo] = useState<AppUpdateInfo | null>(null);
  const [checkingUpdate, setCheckingUpdate] = useState(false);
  const [loggingOut, setLoggingOut] = useState(false);
  const [logoutError, setLogoutError] = useState<string | null>(null);
  const [confirmingLogout, setConfirmingLogout] = useState(false);

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
    setLoggingOut(true);
    setLogoutError(null);
    try {
      await authLogout();
      onLogout();
    } catch (err) {
      console.error("Failed to logout:", err);
      setLogoutError(err instanceof Error ? err.message : String(err));
      setLoggingOut(false);
      setConfirmingLogout(false);
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
          {confirmingLogout ? (
            <div className="space-y-2">
              <p className="text-xs text-zinc-400">Are you sure you want to sign out?</p>
              <div className="flex gap-2">
                <button
                  type="button"
                  onClick={handleLogout}
                  disabled={loggingOut}
                  className="flex-1 text-xs px-3 py-2 rounded-md bg-red-900/30 hover:bg-red-900/50 disabled:opacity-50 text-red-400 border border-red-800/50 transition-colors"
                >
                  {loggingOut ? "Signing out..." : "Yes, Sign Out"}
                </button>
                <button
                  type="button"
                  onClick={() => setConfirmingLogout(false)}
                  disabled={loggingOut}
                  className="flex-1 text-xs px-3 py-2 rounded-md bg-zinc-700 hover:bg-zinc-600 disabled:opacity-50 text-zinc-300 transition-colors"
                >
                  Cancel
                </button>
              </div>
            </div>
          ) : (
            <button
              type="button"
              onClick={() => setConfirmingLogout(true)}
              className="w-full text-xs px-3 py-2 rounded-md bg-red-900/30 hover:bg-red-900/50 text-red-400 border border-red-800/50 transition-colors"
            >
              Sign Out
            </button>
          )}
          {logoutError && (
            <p className="text-xs text-red-400">Failed to sign out: {logoutError}</p>
          )}
        </div>
      </div>
    </div>
  );
}
