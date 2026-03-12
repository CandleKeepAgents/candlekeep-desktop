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

export interface PluginStatus {
  installed: boolean;
  version: string | null;
  path: string | null;
}

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

export interface AppUpdateInfo {
  update_available: boolean;
  current_version: string;
  latest_version: string | null;
  download_url: string | null;
  dmg_url: string | null;
  checksum_url: string | null;
}

export type SetupStep =
  | "welcome"
  | "system-check"
  | "install-homebrew"
  | "install-cli"
  | "authenticate"
  | "install-claude"
  | "install-plugin"
  | "done";

export interface SystemCheckResult {
  homebrew: boolean;
  cargo: boolean;
  node: boolean;
  xcodeClt: boolean;
  cliInstalled: boolean;
  cliVersion: string | null;
  authenticated: boolean;
  claudeCodeInstalled: boolean;
  pluginInstalled: boolean;
  pluginVersion: string | null;
}
