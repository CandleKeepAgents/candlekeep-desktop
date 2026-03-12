import { useCallback, useEffect, useRef, useState } from "react";
import { checkAuthStatus, checkCliInstalled, getLatestCliVersion } from "../lib/tauri-commands";
import type { AuthStatus, CliStatus } from "../lib/types";

const MAX_INTERVAL = 5 * 60 * 1000; // 5 minutes

export function useCliStatus(pollInterval = 10000) {
  const [cliStatus, setCliStatus] = useState<CliStatus | null>(null);
  const [authStatus, setAuthStatus] = useState<AuthStatus | null>(null);
  const [latestVersion, setLatestVersion] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const failureCount = useRef(0);

  const refresh = useCallback(async () => {
    try {
      const [cli, auth, latest] = await Promise.all([
        checkCliInstalled(),
        checkAuthStatus(),
        getLatestCliVersion(),
      ]);
      setCliStatus(cli);
      setAuthStatus(auth);
      setLatestVersion(latest);
      failureCount.current = 0;
    } catch (err) {
      console.error("Failed to check CLI status:", err);
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

  const updateAvailable =
    cliStatus?.version && latestVersion
      ? cliStatus.version !== latestVersion
      : false;

  return { cliStatus, authStatus, latestVersion, updateAvailable, loading, refresh };
}
