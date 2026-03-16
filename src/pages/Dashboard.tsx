import { openUrl } from "@tauri-apps/plugin-opener";
import { useCallback, useEffect, useRef, useState } from "react";
import { MetricsCard } from "../components/MetricsCard";
import { QuickActions } from "../components/QuickActions";
import { StatusCard } from "../components/StatusCard";
import { UpdateBanner } from "../components/UpdateBanner";
import { useCliStatus } from "../hooks/useCliStatus";
import { useIntegrations } from "../hooks/useIntegrations";
import { useMetrics } from "../hooks/useMetrics";
import {
  checkAppUpdate,
  checkAuthStatus,
  installAppUpdate,
  triggerAuthLogin,
  updateCli,
} from "../lib/tauri-commands";
import type { AppUpdateInfo } from "../lib/types";
import { HOST_DISPLAY_NAMES } from "../lib/types";

export function Dashboard() {
  const {
    cliStatus,
    authStatus,
    latestVersion,
    updateAvailable,
    loading: cliLoading,
    refresh: refreshCli,
  } = useCliStatus();
  const {
    integrations,
    loading: integrationsLoading,
    refresh: refreshIntegrations,
  } = useIntegrations();
  const {
    metrics,
    loading: metricsLoading,
    refresh: refreshMetrics,
  } = useMetrics();
  const [checkingUpdates, setCheckingUpdates] = useState(false);
  const [appUpdate, setAppUpdate] = useState<AppUpdateInfo | null>(null);
  const [updatingApp, setUpdatingApp] = useState(false);
  const [authenticating, setAuthenticating] = useState(false);
  const authPollRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const authTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const refreshAppUpdate = useCallback(async () => {
    try {
      const info = await checkAppUpdate();
      setAppUpdate(info);
    } catch (err) {
      console.error("Failed to check app update:", err);
    }
  }, []);

  useEffect(() => {
    refreshAppUpdate();
  }, [refreshAppUpdate]);

  const handleOpenLibrary = async () => {
    try {
      await openUrl("https://www.getcandlekeep.com/dashboard");
    } catch (err) {
      console.error("Failed to open library:", err);
    }
  };

  const handleJoinSlack = async () => {
    try {
      await openUrl("https://join.slack.com/t/candlekeepgroup/shared_invite/zt-3sxxll6kg-McekCEeUQJz6Eh2xqS7v8A");
    } catch (err) {
      console.error("Failed to open Slack:", err);
    }
  };

  const clearAuthPolling = useCallback(() => {
    if (authPollRef.current) {
      clearInterval(authPollRef.current);
      authPollRef.current = null;
    }
    if (authTimeoutRef.current) {
      clearTimeout(authTimeoutRef.current);
      authTimeoutRef.current = null;
    }
  }, []);

  useEffect(() => {
    return () => clearAuthPolling();
  }, [clearAuthPolling]);

  const handleReAuth = async () => {
    if (authenticating) return;
    setAuthenticating(true);
    clearAuthPolling();
    try {
      await triggerAuthLogin();
      authPollRef.current = setInterval(async () => {
        try {
          const auth = await checkAuthStatus();
          if (auth.authenticated) {
            clearAuthPolling();
            setAuthenticating(false);
            refreshCli();
            refreshMetrics();
          }
        } catch {
          // Ignore poll errors
        }
      }, 2000);
      authTimeoutRef.current = setTimeout(() => {
        clearAuthPolling();
        setAuthenticating(false);
      }, 300000);
    } catch (err) {
      console.error("Failed to trigger auth:", err);
      setAuthenticating(false);
    }
  };

  const handleCheckUpdates = async () => {
    setCheckingUpdates(true);
    try {
      await Promise.all([
        refreshCli(),
        refreshMetrics(),
        refreshAppUpdate(),
        refreshIntegrations(),
      ]);
    } finally {
      setCheckingUpdates(false);
    }
  };

  const handleUpdateCli = async () => {
    try {
      await updateCli();
      refreshCli();
    } catch (err) {
      console.error("Failed to update CLI:", err);
    }
  };

  const handleUpdateApp = async () => {
    if (!appUpdate?.asset_url) {
      if (appUpdate?.download_url) {
        await openUrl(appUpdate.download_url);
      }
      return;
    }
    setUpdatingApp(true);
    try {
      await installAppUpdate(appUpdate.asset_url);
    } catch (err) {
      console.error("Failed to install app update:", err);
      if (appUpdate?.download_url) {
        await openUrl(appUpdate.download_url);
      }
    } finally {
      setUpdatingApp(false);
    }
  };

  return (
    <div className="space-y-4">
      <h2 className="text-lg font-semibold text-zinc-100">Dashboard</h2>

      {appUpdate?.update_available && appUpdate.latest_version && (
        <UpdateBanner
          label="App Update"
          currentVersion={appUpdate.current_version}
          latestVersion={appUpdate.latest_version}
          onUpdate={handleUpdateApp}
          loading={updatingApp}
        />
      )}

      {updateAvailable && cliStatus?.version && latestVersion && (
        <UpdateBanner
          label="CLI Update"
          currentVersion={cliStatus.version}
          latestVersion={latestVersion}
          onUpdate={handleUpdateCli}
        />
      )}

      <div className="space-y-2">
        <StatusCard
          title="CandleKeep CLI"
          status={
            cliLoading
              ? "loading"
              : cliStatus?.installed
                ? "ok"
                : "error"
          }
          detail={
            cliStatus?.installed
              ? `v${cliStatus.version} (${cliStatus.install_method})`
              : "Not installed"
          }
        />
        <StatusCard
          title="Authentication"
          status={
            cliLoading
              ? "loading"
              : authenticating
                ? "loading"
                : authStatus?.authenticated
                  ? "ok"
                  : "error"
          }
          detail={
            authenticating
              ? "Waiting for authentication..."
              : authStatus?.authenticated
                ? "Signed in"
                : "Not authenticated"
          }
          action={
            !authStatus?.authenticated && !authenticating
              ? { label: "Sign In", onClick: handleReAuth }
              : undefined
          }
        />

        {/* Integration status cards */}
        {integrations.map((integration) => (
          <StatusCard
            key={integration.host}
            title={HOST_DISPLAY_NAMES[integration.host]}
            status={
              integrationsLoading
                ? "loading"
                : integration.integration_installed
                  ? "ok"
                  : integration.host_installed
                    ? "warning"
                    : "warning"
            }
            detail={
              integration.integration_installed
                ? integration.version
                  ? `v${integration.version}`
                  : "Installed"
                : integration.host_installed
                  ? "Not configured"
                  : "Not found"
            }
          />
        ))}
      </div>

      <MetricsCard
        whoami={metrics?.whoami ?? null}
        loading={metricsLoading}
      />

      <QuickActions
        onOpenLibrary={handleOpenLibrary}
        onReAuth={handleReAuth}
        onCheckUpdates={handleCheckUpdates}
        onJoinSlack={handleJoinSlack}
        updatesLoading={checkingUpdates}
        authLoading={authenticating}
      />
    </div>
  );
}
