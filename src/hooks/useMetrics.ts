import { useCallback, useEffect, useRef, useState } from "react";
import { fetchMetrics } from "../lib/tauri-commands";
import type { Metrics } from "../lib/types";

const MAX_INTERVAL = 5 * 60 * 1000; // 5 minutes

export function useMetrics(pollInterval = 30000) {
  const [metrics, setMetrics] = useState<Metrics | null>(null);
  const [loading, setLoading] = useState(true);
  const failureCount = useRef(0);

  const refresh = useCallback(async () => {
    try {
      const data = await fetchMetrics();
      setMetrics(data);
      failureCount.current = 0;
    } catch (err) {
      console.error("Failed to fetch metrics:", err);
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

  return { metrics, loading, refresh };
}
