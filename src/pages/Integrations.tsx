import { IntegrationCard } from "../components/IntegrationCard";
import { usePluginStatus } from "../hooks/usePluginStatus";
import { installPlugin, updatePlugin } from "../lib/tauri-commands";

export function Integrations() {
  const { pluginStatus, claudeCodeInstalled, refresh } = usePluginStatus();

  const handleInstallPlugin = async () => {
    try {
      await installPlugin();
      refresh();
    } catch (err) {
      console.error("Failed to install plugin:", err);
    }
  };

  const handleUpdatePlugin = async () => {
    try {
      await updatePlugin();
      refresh();
    } catch (err) {
      console.error("Failed to update plugin:", err);
    }
  };

  const claudeCodeStatus = pluginStatus?.installed
    ? "installed"
    : claudeCodeInstalled
    ? "available"
    : "available";

  return (
    <div className="space-y-4">
      <h2 className="text-lg font-semibold text-zinc-100">Integrations</h2>
      <p className="text-xs text-zinc-400">
        Manage CandleKeep integrations with your AI coding tools.
      </p>

      <div className="space-y-3">
        <IntegrationCard
          name="Claude Code"
          description="Access your CandleKeep library directly from Claude Code with AI-powered search and document management."
          installed={pluginStatus?.installed ?? false}
          version={pluginStatus?.version}
          status={claudeCodeStatus as "installed" | "available" | "update-available" | "coming-soon"}
          onInstall={handleInstallPlugin}
          onUpdate={handleUpdatePlugin}
        />

        <IntegrationCard
          name="Cursor"
          description="Use your CandleKeep library as context in Cursor IDE for smarter code assistance."
          installed={false}
          status="coming-soon"
        />

        <IntegrationCard
          name="Codex"
          description="Connect your CandleKeep library to OpenAI Codex for enhanced coding workflows."
          installed={false}
          status="coming-soon"
        />
      </div>
    </div>
  );
}
