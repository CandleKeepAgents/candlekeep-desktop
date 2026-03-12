use super::{Integration, IntegrationStatus};
use std::path::PathBuf;

pub struct ClaudeCodeIntegration;

impl ClaudeCodeIntegration {
    pub fn new() -> Self {
        Self
    }

    fn plugin_path() -> Option<PathBuf> {
        let home = dirs::home_dir()?;
        let path = home.join(".claude/plugins/marketplaces/candlekeep/plugins/candlekeep-cloud");
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }

    fn claude_code_exists() -> bool {
        let mut paths = vec![
            PathBuf::from("/opt/homebrew/bin/claude"),
            PathBuf::from("/usr/local/bin/claude"),
        ];
        if let Some(home) = dirs::home_dir() {
            paths.push(home.join(".local/bin/claude"));
        }
        paths.iter().any(|p| p.exists())
    }

    fn read_plugin_version() -> Option<String> {
        let path = Self::plugin_path()?;
        let json_path = path.join(".claude-plugin/plugin.json");
        let content = std::fs::read_to_string(json_path).ok()?;
        let json: serde_json::Value = serde_json::from_str(&content).ok()?;
        json.get("version")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }
}

impl Integration for ClaudeCodeIntegration {
    fn name(&self) -> &str {
        "claude-code"
    }

    fn display_name(&self) -> &str {
        "Claude Code"
    }

    fn description(&self) -> &str {
        "CandleKeep plugin for Claude Code — access your library from the AI coding assistant"
    }

    fn is_installed(&self) -> bool {
        Self::plugin_path().is_some()
    }

    fn current_version(&self) -> Option<String> {
        Self::read_plugin_version()
    }

    fn latest_version(&self) -> Option<String> {
        // TODO: fetch from GitHub releases
        None
    }

    fn status(&self) -> IntegrationStatus {
        if self.is_installed() {
            IntegrationStatus::Installed
        } else {
            // Check if Claude Code itself is installed via known paths
            let has_claude = Self::claude_code_exists();
            if has_claude {
                IntegrationStatus::Available
            } else {
                IntegrationStatus::Available
            }
        }
    }
}
