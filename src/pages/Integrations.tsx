import { useState } from "react";
import { IntegrationCard } from "../components/IntegrationCard";
import { useIntegrations } from "../hooks/useIntegrations";
import {
  installIntegration,
  repairIntegration,
  uninstallIntegration,
  updateIntegration,
} from "../lib/tauri-commands";
import type { HostKind } from "../lib/types";
import { HOST_DESCRIPTIONS, HOST_DISPLAY_NAMES } from "../lib/types";

export function Integrations() {
  const { integrations, loading, refresh } = useIntegrations();
  const [actionLoading, setActionLoading] = useState<HostKind | null>(null);
  const [errorMessage, setErrorMessage] = useState<{host: HostKind; message: string} | null>(null);
  const handleAction = async (
    host: HostKind,
    action: "install" | "uninstall" | "update" | "repair",
  ) => {
    setActionLoading(host);
    setErrorMessage(null);
    try {
      const fn =
        action === "install"
          ? installIntegration
          : action === "uninstall"
            ? uninstallIntegration
            : action === "update"
              ? updateIntegration
              : repairIntegration;
      const result = await fn(host);
      if (!result.ok) {
        console.error(`${action} failed:`, result.message);
        setErrorMessage({ host, message: result.message });
        setTimeout(() => setErrorMessage((prev) => prev?.host === host ? null : prev), 5000);
      }
      refresh();
    } catch (err) {
      const message = `Failed to ${action} integration: ${err}`;
      console.error(message);
      setErrorMessage({ host, message });
      setTimeout(() => setErrorMessage((prev) => prev?.host === host ? null : prev), 5000);
    } finally {
      setActionLoading(null);
    }
  };

  const getCardStatus = (
    integration: (typeof integrations)[0],
  ): "installed" | "available" | "update-available" | "coming-soon" => {
    if (integration.integration_installed) {
      if (
        integration.latest_version &&
        integration.version &&
        integration.latest_version !== integration.version
      ) {
        return "update-available";
      }
      return "installed";
    }
    if (integration.host_installed) {
      return "available";
    }
    return "available";
  };

  return (
    <div className="space-y-4">
      <h2 className="text-lg font-semibold text-zinc-100">Integrations</h2>
      <p className="text-xs text-zinc-400">
        Manage CandleKeep integrations with your AI coding tools.
      </p>

      {loading ? (
        <div className="text-center text-sm text-zinc-400 py-8">
          Loading integrations...
        </div>
      ) : (
        <div className="space-y-3">
          {(() => {
            const visible = integrations.filter(i => i.host_installed);
            if (visible.length === 0) {
              return (
                <div className="text-center text-sm text-zinc-400 py-8">
                  No AI coding tools detected. Install one to get started.
                </div>
              );
            }
            return visible.map((integration) => (
              <IntegrationCard
                key={integration.host}
                name={HOST_DISPLAY_NAMES[integration.host]}
                description={HOST_DESCRIPTIONS[integration.host]}
                installed={integration.integration_installed}
                version={integration.version}
                status={getCardStatus(integration)}
                loading={actionLoading === integration.host}
                error={errorMessage?.host === integration.host ? errorMessage.message : undefined}
                onInstall={() => handleAction(integration.host, "install")}
                onUninstall={() => handleAction(integration.host, "uninstall")}
                onUpdate={() => handleAction(integration.host, "update")}
              />
            ));
          })()}
        </div>
      )}
    </div>
  );
}
