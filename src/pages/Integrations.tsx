import { IntegrationCard } from "../components/IntegrationCard";
import { useIntegrations } from "../hooks/useIntegrations";
import {
  installIntegration,
  updateIntegration,
  repairIntegration,
} from "../lib/tauri-commands";
import type { HostKind } from "../lib/types";
import { HOST_DISPLAY_NAMES, HOST_DESCRIPTIONS } from "../lib/types";

export function Integrations() {
  const { integrations, loading, refresh } = useIntegrations();
  const handleAction = async (
    host: HostKind,
    action: "install" | "update" | "repair",
  ) => {
    try {
      const fn =
        action === "install"
          ? installIntegration
          : action === "update"
            ? updateIntegration
            : repairIntegration;
      const result = await fn(host);
      if (!result.ok) {
        console.error(`${action} failed:`, result.message);
      }
      refresh();
    } catch (err) {
      console.error(`Failed to ${action} integration:`, err);
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
          {integrations.map((integration) => (
            <IntegrationCard
              key={integration.host}
              name={HOST_DISPLAY_NAMES[integration.host]}
              description={HOST_DESCRIPTIONS[integration.host]}
              installed={integration.integration_installed}
              version={integration.version}
              status={getCardStatus(integration)}
              onInstall={() => handleAction(integration.host, "install")}
              onUpdate={() => handleAction(integration.host, "update")}
            />
          ))}
        </div>
      )}
    </div>
  );
}
