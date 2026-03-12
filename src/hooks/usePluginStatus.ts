import { useCallback, useEffect, useRef, useState } from "react";
import { checkClaudeCodeInstalled, checkPluginInstalled } from "../lib/tauri-commands";
import type { PluginStatus } from "../lib/types";

const MAX_INTERVAL = 5 * 60 * 1000; // 5 minutes

export function usePluginStatus(pollInterval = 10000) {
  const [pluginStatus, setPluginStatus] = useState<PluginStatus | null>(null);
  const [claudeCodeInstalled, setClaudeCodeInstalled] = useState<boolean | null>(null);
  const [loading, setLoading] = useState(true);
  const failureCount = useRef(0);

  const refresh = useCallback(async () => {
    try {
      const [plugin, claude] = await Promise.all([
        checkPluginInstalled(),
        checkClaudeCodeInstalled(),
      ]);
      setPluginStatus(plugin);
      setClaudeCodeInstalled(claude);
      failureCount.current = 0;
    } catch (err) {
      console.error("Failed to check plugin status:", err);
      failureCount.current += 1;
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
    let timeoutId: ReturnType<typeof setTimeout>;

    const schedule = () => {
      const backoff = Math.min(
        pollInterval * 2 ** failureCount.current,
        MAX_INTERVAL
      );
      timeoutId = setTimeout(() => {
        refresh().then(schedule);
      }, backoff);
    };

    schedule();
    return () => clearTimeout(timeoutId);
  }, [refresh, pollInterval]);

  return { pluginStatus, claudeCodeInstalled, loading, refresh };
}
