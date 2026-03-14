import { invoke } from "@tauri-apps/api/core";
import type {
  ActionResult,
  AppUpdateInfo,
  AuthStatus,
  CliStatus,
  HostKind,
  IntegrationStatus,
  Metrics,
  PlatformInfo,
  PluginStatus,
  WhoamiResponse,
} from "./types";

// --- Platform commands ---
export const getPlatformInfo = () => invoke<PlatformInfo>("get_platform_info");

// --- System commands ---
export const checkHomebrew = () => invoke<boolean>("check_homebrew");
export const checkCargo = () => invoke<boolean>("check_cargo");
export const checkNode = () => invoke<boolean>("check_node");
export const checkXcodeClt = () => invoke<boolean>("check_xcode_clt");
export const installHomebrew = () => invoke<string>("install_homebrew");

// --- CLI commands ---
export const checkCliInstalled = () => invoke<CliStatus>("check_cli_installed");
export const getCliVersion = () => invoke<string | null>("get_cli_version");
export const getLatestCliVersion = () =>
  invoke<string | null>("get_latest_cli_version");
export const installCli = () => invoke<string>("install_cli");
export const updateCli = () => invoke<string>("update_cli");
export const checkAuthStatus = () => invoke<AuthStatus>("check_auth_status");
export const triggerAuthLogin = () => invoke<string>("trigger_auth_login");
export const authLogout = () => invoke<string>("auth_logout");

// --- Generic integration commands ---
export const listIntegrations = () =>
  invoke<IntegrationStatus[]>("list_integrations");
export const checkIntegration = (host: HostKind) =>
  invoke<IntegrationStatus>("check_integration", { host });
export const installIntegration = (host: HostKind) =>
  invoke<ActionResult>("install_integration", { host });
export const updateIntegration = (host: HostKind) =>
  invoke<ActionResult>("update_integration", { host });
export const repairIntegration = (host: HostKind) =>
  invoke<ActionResult>("repair_integration", { host });

// --- Legacy plugin commands (backward compatibility shims) ---
export const checkPluginInstalled = () =>
  invoke<PluginStatus>("check_plugin_installed");
export const getPluginVersion = () =>
  invoke<string | null>("get_plugin_version");
export const installPlugin = () => invoke<string>("install_plugin");
export const updatePlugin = () => invoke<string>("update_plugin");
export const checkClaudeCodeInstalled = () =>
  invoke<boolean>("check_claude_code_installed");

// --- Metrics commands ---
export const fetchWhoami = () => invoke<WhoamiResponse>("fetch_whoami");
export const fetchMetrics = () => invoke<Metrics>("fetch_metrics");

// --- Updater commands ---
export const checkAppUpdate = () =>
  invoke<AppUpdateInfo>("check_app_update");
export const installAppUpdate = (
  assetUrl: string,
  expectedChecksum?: string | null,
) =>
  invoke<string>("install_app_update", {
    assetUrl,
    expectedChecksum: expectedChecksum ?? null,
  });
