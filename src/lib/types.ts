// --- Platform types ---

export type Platform = "macos" | "windows" | "linux";

export interface PlatformPaths {
  cli_install_dir: string;
  config_dir: string;
  extra_bin_dirs: string[];
  path_separator: string;
}

export interface PlatformInfo {
  platform: Platform;
  arch: string;
  tray_supported: boolean;
  paths: PlatformPaths;
}

// --- CLI types ---

export interface CliStatus {
  installed: boolean;
  version: string | null;
  path: string | null;
  install_method: string | null;
}

export interface AuthStatus {
  authenticated: boolean;
  api_key_present: boolean;
}

// --- Integration types ---

export type HostKind = "claude_code" | "cursor" | "codex" | "amp";

export type RequirementStatus = "satisfied" | "missing" | "unsupported";

export interface IntegrationStatus {
  host: HostKind;
  host_installed: boolean;
  integration_installed: boolean;
  version: string | null;
  latest_version: string | null;
  install_method: string;
  status: RequirementStatus;
}

export interface ActionResult {
  ok: boolean;
  message: string;
  details: string | null;
  restart_required: boolean;
}

// --- Legacy plugin types (kept for backward compatibility) ---

export interface PluginStatus {
  installed: boolean;
  version: string | null;
  path: string | null;
}

// --- Metrics types ---

export interface WhoamiResponse {
  id: string;
  email: string;
  name: string | null;
  tier: string;
  item_limit: number | null;
  item_count: number | null;
}

export interface Metrics {
  whoami: WhoamiResponse | null;
  error: string | null;
}

// --- Updater types ---

export interface AppUpdateInfo {
  update_available: boolean;
  current_version: string;
  latest_version: string | null;
  download_url: string | null;
  asset_url: string | null;
  checksum_url: string | null;
}

// --- Setup types ---

export type SetupStep =
  | "welcome"
  | "host-picker"
  | "system-check"
  | "install-homebrew"
  | "install-cli"
  | "authenticate"
  | "install-host"
  | "install-integration"
  | "done";

export interface SystemCheckResult {
  platform: PlatformInfo | null;
  homebrew: boolean;
  cargo: boolean;
  node: boolean;
  xcodeClt: boolean;
  cliInstalled: boolean;
  cliVersion: string | null;
  authenticated: boolean;
  integrations: IntegrationStatus[];
}

// --- Host display helpers ---

export const HOST_DISPLAY_NAMES: Record<HostKind, string> = {
  claude_code: "Claude Code",
  cursor: "Cursor",
  codex: "Codex",
  amp: "Amp",
};

export const HOST_DESCRIPTIONS: Record<HostKind, string> = {
  claude_code:
    "Access your CandleKeep library directly from Claude Code with AI-powered search and document management.",
  cursor:
    "Use your CandleKeep library as context in Cursor IDE for smarter code assistance.",
  codex:
    "Connect your CandleKeep library to OpenAI Codex for enhanced coding workflows.",
  amp: "Access your CandleKeep library from Amp for AI-powered development.",
};
