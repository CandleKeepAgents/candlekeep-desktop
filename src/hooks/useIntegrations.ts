import { useCallback, useEffect, useRef, useState } from "react";
import { listIntegrations } from "../lib/tauri-commands";
import type { IntegrationStatus } from "../lib/types";

const MAX_INTERVAL = 5 * 60 * 1000;

export function useIntegrations(pollInterval = 30000) {
  const [integrations, setIntegrations] = useState<IntegrationStatus[]>([]);
  const [loading, setLoading] = useState(true);
  const failureCount = useRef(0);

  const refresh = useCallback(async () => {
    try {
      const result = await listIntegrations();
      setIntegrations(result);
      failureCount.current = 0;
    } catch (err) {
      console.error("Failed to list integrations:", err);
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
        MAX_INTERVAL,
      );
      timeoutId = setTimeout(() => {
        refresh().then(schedule);
      }, backoff);
    };

    schedule();
    return () => clearTimeout(timeoutId);
  }, [refresh, pollInterval]);

  return { integrations, loading, refresh };
}
