import { invoke } from "@tauri-apps/api/core";
import type {
  AppUpdateInfo,
  AuthStatus,
  CliStatus,
  Metrics,
  PluginStatus,
  WhoamiResponse,
} from "./types";

// System commands
export const checkHomebrew = () => invoke<boolean>("check_homebrew");
export const checkCargo = () => invoke<boolean>("check_cargo");
export const checkNode = () => invoke<boolean>("check_node");
export const checkXcodeClt = () => invoke<boolean>("check_xcode_clt");
export const installHomebrew = () => invoke<string>("install_homebrew");

// CLI commands
export const checkCliInstalled = () => invoke<CliStatus>("check_cli_installed");
export const getCliVersion = () => invoke<string | null>("get_cli_version");
export const getLatestCliVersion = () =>
  invoke<string | null>("get_latest_cli_version");
export const installCli = () => invoke<string>("install_cli");
export const updateCli = () => invoke<string>("update_cli");
export const checkAuthStatus = () => invoke<AuthStatus>("check_auth_status");
export const triggerAuthLogin = () => invoke<string>("trigger_auth_login");
export const authLogout = () => invoke<string>("auth_logout");

// Plugin commands
export const checkPluginInstalled = () =>
  invoke<PluginStatus>("check_plugin_installed");
export const getPluginVersion = () =>
  invoke<string | null>("get_plugin_version");
export const installPlugin = () => invoke<string>("install_plugin");
export const updatePlugin = () => invoke<string>("update_plugin");
export const checkClaudeCodeInstalled = () =>
  invoke<boolean>("check_claude_code_installed");

// Metrics commands
export const fetchWhoami = () => invoke<WhoamiResponse>("fetch_whoami");
export const fetchMetrics = () => invoke<Metrics>("fetch_metrics");

// Updater commands
export const checkAppUpdate = () =>
  invoke<AppUpdateInfo>("check_app_update");
export const installAppUpdate = (dmgUrl: string, expectedChecksum?: string | null) =>
  invoke<string>("install_app_update", { dmgUrl, expectedChecksum: expectedChecksum ?? null });
