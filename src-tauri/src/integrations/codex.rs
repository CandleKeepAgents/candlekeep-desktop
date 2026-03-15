use std::path::PathBuf;
use tracing::info;

use crate::platform::installer::ActionResult;
use crate::platform::paths;
use crate::platform::PlatformInfo;

use super::{HostIntegration, HostKind, IntegrationStatus, Requirement, RequirementStatus};

pub struct CodexAdapter;

impl CodexAdapter {
    pub fn new() -> Self {
        Self
    }

    /// Get the Codex config directory per platform.
    fn config_dir() -> Option<PathBuf> {
        let home = dirs::home_dir()?;
        // Codex uses ~/.codex/ on all platforms
        Some(home.join(".codex"))
    }

    fn is_codex_installed() -> bool {
        let info = PlatformInfo::detect();
        paths::find_binary("codex", &info).is_some()
    }

    /// Path to the Codex config.toml file.
    fn config_toml_path() -> Option<PathBuf> {
        Self::config_dir().map(|d| d.join("config.toml"))
    }

    /// Check if CandleKeep MCP server is configured in config.toml under [mcp_servers.candlekeep].
    fn is_mcp_configured() -> bool {
        let Some(path) = Self::config_toml_path() else { return false };
        if !path.exists() { return false; }

        let Ok(content) = std::fs::read_to_string(&path) else { return false };
        let Ok(config) = content.parse::<toml::Table>() else { return false };

        config.get("mcp_servers")
            .and_then(|s| s.as_table())
            .and_then(|s| s.get("candlekeep"))
            .is_some()
    }

    /// Write CandleKeep MCP server entry into config.toml under [mcp_servers.candlekeep].
    fn write_mcp_config() -> Result<(), String> {
        let info = PlatformInfo::detect();
        let ck_path = paths::find_binary("ck", &info)
            .ok_or("CandleKeep CLI (ck) not found. Install it first.")?;

        let path = Self::config_toml_path()
            .ok_or("Could not determine Codex config path")?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config dir: {}", e))?;
        }

        let mut config: toml::Table = if path.exists() {
            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read config.toml: {}", e))?;
            content.parse::<toml::Table>()
                .unwrap_or_else(|_| toml::Table::new())
        } else {
            toml::Table::new()
        };

        // Build the candlekeep MCP server entry
        let mut server_entry = toml::Table::new();
        server_entry.insert(
            "command".to_string(),
            toml::Value::String(ck_path.to_string_lossy().to_string()),
        );
        server_entry.insert(
            "args".to_string(),
            toml::Value::Array(vec![
                toml::Value::String("mcp".to_string()),
                toml::Value::String("serve".to_string()),
            ]),
        );

        // Add env block so spawned process can find ~/.candlekeep/config.toml
        if let Some(home) = dirs::home_dir() {
            let mut env_table = toml::Table::new();
            env_table.insert(
                "HOME".to_string(),
                toml::Value::String(home.to_string_lossy().to_string()),
            );
            server_entry.insert("env".to_string(), toml::Value::Table(env_table));
        }

        // Get or create mcp_servers table
        let mcp_servers = config
            .entry("mcp_servers")
            .or_insert_with(|| toml::Value::Table(toml::Table::new()));

        if let Some(servers) = mcp_servers.as_table_mut() {
            servers.insert(
                "candlekeep".to_string(),
                toml::Value::Table(server_entry),
            );
        }

        let output = toml::to_string_pretty(&config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        std::fs::write(&path, output)
            .map_err(|e| format!("Failed to write config: {}", e))?;

        Ok(())
    }

    /// Remove the old mcp.json entry if it exists (migration from previous format).
    fn cleanup_old_mcp_json() {
        if let Some(dir) = Self::config_dir() {
            let old_path = dir.join("mcp.json");
            if old_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&old_path) {
                    if let Ok(mut json) = serde_json::from_str::<serde_json::Value>(&content) {
                        // Remove only the candlekeep entry, leave others intact
                        if let Some(servers) = json.get_mut("mcpServers").and_then(|s| s.as_object_mut()) {
                            servers.remove("candlekeep");
                            // If no servers left, delete the file; otherwise rewrite it
                            if servers.is_empty() {
                                let _ = std::fs::remove_file(&old_path);
                            } else if let Ok(output) = serde_json::to_string_pretty(&json) {
                                let _ = std::fs::write(&old_path, output);
                            }
                        }
                    }
                }
            }
        }
    }
}

impl HostIntegration for CodexAdapter {
    fn kind(&self) -> HostKind {
        HostKind::Codex
    }

    fn detect_host(&self, _platform: &PlatformInfo) -> bool {
        Self::is_codex_installed()
    }

    fn detect_integration(&self) -> IntegrationStatus {
        let host_installed = Self::is_codex_installed();
        let integration_installed = Self::is_mcp_configured();

        let status = if !host_installed {
            RequirementStatus::Missing
        } else if integration_installed {
            RequirementStatus::Satisfied
        } else {
            RequirementStatus::Missing
        };

        IntegrationStatus {
            host: HostKind::Codex,
            host_installed,
            integration_installed,
            version: None,
            latest_version: None,
            install_method: "mcp-config".to_string(),
            status,
        }
    }

    fn uninstall(&self) -> ActionResult {
        let Some(path) = Self::config_toml_path() else {
            return ActionResult::failure("Could not determine Codex config path");
        };
        if !path.exists() {
            return ActionResult::success("Nothing to uninstall");
        }
        let Ok(content) = std::fs::read_to_string(&path) else {
            return ActionResult::failure("Failed to read config.toml");
        };
        let Ok(mut config) = content.parse::<toml::Table>() else {
            return ActionResult::failure("Failed to parse config.toml");
        };
        if let Some(servers) = config.get_mut("mcp_servers").and_then(|s| s.as_table_mut()) {
            servers.remove("candlekeep");
        }
        match toml::to_string_pretty(&config) {
            Ok(output) => {
                if let Err(e) = std::fs::write(&path, output) {
                    return ActionResult::failure(format!("Failed to write config: {}", e));
                }
                info!("CandleKeep MCP server removed from Codex");
                let mut result = ActionResult::success("CandleKeep removed from Codex");
                result.restart_required = true;
                result
            }
            Err(e) => ActionResult::failure(format!("Failed to serialize config: {}", e)),
        }
    }

    fn install(&self) -> ActionResult {
        if !Self::is_codex_installed() {
            return ActionResult::failure("Codex is not installed. Install Codex first.");
        }

        match Self::write_mcp_config() {
            Ok(()) => {
                // Clean up the old mcp.json entry if present
                Self::cleanup_old_mcp_json();
                info!("CandleKeep MCP server configured for Codex");
                let mut result = ActionResult::success("CandleKeep configured for Codex");
                result.restart_required = true;
                result
            }
            Err(e) => ActionResult::failure(e),
        }
    }

    fn update(&self) -> ActionResult {
        self.install()
    }

    fn repair(&self) -> ActionResult {
        self.install()
    }

    fn requirements(&self, platform: &PlatformInfo) -> Vec<Requirement> {
        vec![
            Requirement {
                name: "Codex".to_string(),
                description: "Install OpenAI Codex CLI".to_string(),
                status: if Self::is_codex_installed() {
                    RequirementStatus::Satisfied
                } else {
                    RequirementStatus::Missing
                },
            },
            Requirement {
                name: "CandleKeep CLI".to_string(),
                description: "Required for MCP server".to_string(),
                status: if paths::find_binary("ck", platform).is_some() {
                    RequirementStatus::Satisfied
                } else {
                    RequirementStatus::Missing
                },
            },
        ]
    }
}
