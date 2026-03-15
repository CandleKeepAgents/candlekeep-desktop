use std::path::PathBuf;
use std::process::Command;
use tracing::{info, warn, error, debug};

use crate::platform::installer::ActionResult;
use crate::platform::paths;
use crate::platform::PlatformInfo;

use super::{HostIntegration, HostKind, IntegrationStatus, Requirement, RequirementStatus};

pub struct ClaudeCodeAdapter;

impl ClaudeCodeAdapter {
    pub fn new() -> Self {
        Self
    }

    fn find_claude_binary() -> Option<PathBuf> {
        let info = PlatformInfo::detect();
        paths::find_binary("claude", &info)
    }

    fn get_plugin_path() -> Option<PathBuf> {
        let home = dirs::home_dir()?;

        // Primary: read install path from installed_plugins.json
        let installed_json_path = home.join(".claude/plugins/installed_plugins.json");
        if let Ok(content) = std::fs::read_to_string(&installed_json_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(entries) = json
                    .get("plugins")
                    .and_then(|p| p.get("candlekeep-cloud@candlekeep"))
                    .and_then(|e| e.as_array())
                {
                    for entry in entries {
                        if let Some(install_path) = entry.get("installPath").and_then(|p| p.as_str()) {
                            let path = PathBuf::from(install_path);
                            if path.exists() {
                                debug!("Found plugin via installed_plugins.json: {}", path.display());
                                return Some(path);
                            }
                        }
                    }
                }
            }
        }

        // Fallback: check known paths
        let candidates = [
            home.join(".claude/plugins/cache/candlekeep/candlekeep-cloud"),
            home.join(".claude/plugins/marketplaces/candlekeep/plugins/candlekeep-cloud"),
        ];

        for base in &candidates {
            if base.exists() {
                if let Ok(entries) = std::fs::read_dir(base) {
                    let mut versions: Vec<PathBuf> = entries
                        .filter_map(|e| e.ok())
                        .map(|e| e.path())
                        .filter(|p| p.is_dir())
                        .collect();
                    versions.sort();
                    if let Some(latest) = versions.last() {
                        debug!("Found plugin via filesystem scan: {}", latest.display());
                        return Some(latest.clone());
                    }
                }
                if base.join(".claude-plugin/plugin.json").exists() {
                    debug!("Found plugin at base path: {}", base.display());
                    return Some(base.clone());
                }
            }
        }

        warn!("CandleKeep plugin not found");
        None
    }

    fn read_plugin_version(plugin_path: &PathBuf) -> Option<String> {
        let plugin_json_path = plugin_path.join(".claude-plugin/plugin.json");
        let content = std::fs::read_to_string(plugin_json_path).ok()?;
        let json: serde_json::Value = serde_json::from_str(&content).ok()?;
        json.get("version")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }
}

impl HostIntegration for ClaudeCodeAdapter {
    fn kind(&self) -> HostKind {
        HostKind::ClaudeCode
    }

    fn detect_host(&self, _platform: &PlatformInfo) -> bool {
        Self::find_claude_binary().is_some()
    }

    fn detect_integration(&self) -> IntegrationStatus {
        let info = PlatformInfo::detect();
        let host_installed = self.detect_host(&info);
        let plugin_path = Self::get_plugin_path();
        let integration_installed = plugin_path.is_some();
        let version = plugin_path.as_ref().and_then(Self::read_plugin_version);

        let status = if integration_installed {
            RequirementStatus::Satisfied
        } else if host_installed {
            RequirementStatus::Missing
        } else {
            RequirementStatus::Missing
        };

        IntegrationStatus {
            host: HostKind::ClaudeCode,
            host_installed,
            integration_installed,
            version,
            latest_version: None,
            install_method: "plugin-marketplace".to_string(),
            status,
        }
    }

    fn install(&self) -> ActionResult {
        let info = PlatformInfo::detect();
        let path_env = paths::get_full_path(&info);

        // Step 1: Add marketplace
        let output1 = match Command::new("sh")
            .arg("-c")
            .arg("claude /plugin marketplace add CandleKeepAgents/candlekeep-marketplace")
            .env("PATH", &path_env)
            .output()
        {
            Ok(o) => o,
            Err(e) => return ActionResult::failure(format!("Failed to add marketplace: {}", e)),
        };

        if !output1.status.success() {
            let stderr = String::from_utf8_lossy(&output1.stderr);
            error!("Failed to add marketplace: {}", stderr);
            return ActionResult::failure(format!("Failed to add marketplace: {}", stderr));
        }

        // Step 2: Install plugin
        let output2 = match Command::new("sh")
            .arg("-c")
            .arg("claude /plugin install candlekeep-cloud@candlekeep")
            .env("PATH", &path_env)
            .output()
        {
            Ok(o) => o,
            Err(e) => return ActionResult::failure(format!("Failed to install plugin: {}", e)),
        };

        if !output2.status.success() {
            let stderr = String::from_utf8_lossy(&output2.stderr);
            error!("Plugin installation failed: {}", stderr);
            return ActionResult::failure(format!("Plugin installation failed: {}", stderr));
        }

        match Self::get_plugin_path() {
            Some(path) => {
                info!("Plugin installed successfully at: {}", path.display());
                ActionResult::success("Plugin installed successfully")
            }
            None => {
                error!("Plugin install command succeeded but plugin path not found");
                ActionResult::failure("Plugin install appeared to succeed but plugin files were not found")
            }
        }
    }

    fn uninstall(&self) -> ActionResult {
        let info = PlatformInfo::detect();
        let path_env = paths::get_full_path(&info);

        let output = match Command::new("sh")
            .arg("-c")
            .arg("claude /plugin uninstall candlekeep-cloud@candlekeep")
            .env("PATH", &path_env)
            .output()
        {
            Ok(o) => o,
            Err(e) => return ActionResult::failure(format!("Failed to uninstall plugin: {}", e)),
        };

        if output.status.success() {
            info!("Plugin uninstalled successfully");
            ActionResult::success("Plugin uninstalled successfully")
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("Plugin uninstall failed: {}", stderr);
            ActionResult::failure(format!("Plugin uninstall failed: {}", stderr))
        }
    }

    fn update(&self) -> ActionResult {
        let info = PlatformInfo::detect();
        let path_env = paths::get_full_path(&info);

        let output = match Command::new("sh")
            .arg("-c")
            .arg("claude /plugin marketplace add CandleKeepAgents/candlekeep-marketplace")
            .env("PATH", &path_env)
            .output()
        {
            Ok(o) => o,
            Err(e) => return ActionResult::failure(format!("Failed to update plugin: {}", e)),
        };

        if output.status.success() {
            info!("Plugin updated successfully");
            ActionResult::success("Plugin updated successfully")
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("Plugin update failed: {}", stderr);
            ActionResult::failure(format!("Plugin update failed: {}", stderr))
        }
    }

    fn repair(&self) -> ActionResult {
        // Repair = uninstall + reinstall
        self.install()
    }

    fn requirements(&self, platform: &PlatformInfo) -> Vec<Requirement> {
        let reqs = vec![
            Requirement {
                name: "Node.js".to_string(),
                description: "Required for Claude Code".to_string(),
                status: if paths::find_binary("node", platform).is_some() {
                    RequirementStatus::Satisfied
                } else {
                    RequirementStatus::Missing
                },
            },
            Requirement {
                name: "Claude Code".to_string(),
                description: "npm install -g @anthropic-ai/claude-code".to_string(),
                status: if self.detect_host(platform) {
                    RequirementStatus::Satisfied
                } else {
                    RequirementStatus::Missing
                },
            },
        ];
        reqs
    }
}
