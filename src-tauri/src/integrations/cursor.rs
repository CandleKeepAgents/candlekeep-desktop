use std::path::PathBuf;
use tracing::info;

use crate::platform::installer::ActionResult;
use crate::platform::paths;
use crate::platform::PlatformInfo;

use super::{HostIntegration, HostKind, IntegrationStatus, Requirement, RequirementStatus};

pub struct CursorAdapter;

impl CursorAdapter {
    pub fn new() -> Self {
        Self
    }

    /// Get the Cursor config directory (~/.cursor/) — the standard location
    /// where Cursor reads global MCP config on all platforms.
    fn config_dir() -> Option<PathBuf> {
        dirs::home_dir().map(|h| h.join(".cursor"))
    }

    /// Check if Cursor is installed by looking for its config directory.
    fn is_cursor_installed() -> bool {
        Self::config_dir()
            .map(|d| d.exists())
            .unwrap_or(false)
    }

    /// Path to the MCP config file.
    fn mcp_config_path() -> Option<PathBuf> {
        Self::config_dir().map(|d| d.join("mcp.json"))
    }

    /// Check if CandleKeep MCP server is configured.
    fn is_mcp_configured() -> bool {
        let Some(path) = Self::mcp_config_path() else { return false };
        if !path.exists() { return false; }

        let Ok(content) = std::fs::read_to_string(&path) else { return false };
        let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) else { return false };

        json.get("mcpServers")
            .and_then(|s| s.get("candlekeep"))
            .is_some()
    }

    /// Write the CandleKeep MCP server config into Cursor's mcp.json.
    fn write_mcp_config() -> Result<(), String> {
        let path = Self::mcp_config_path()
            .ok_or("Could not determine Cursor config path")?;

        // Ensure parent dir exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config dir: {}", e))?;
        }

        // Load existing or create new
        let mut config: serde_json::Value = if path.exists() {
            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read MCP config: {}", e))?;
            serde_json::from_str(&content)
                .unwrap_or_else(|_| serde_json::json!({}))
        } else {
            serde_json::json!({})
        };

        // Find the ck binary
        let info = PlatformInfo::detect();
        let ck_path = paths::find_binary("ck", &info)
            .ok_or("CandleKeep CLI (ck) not found. Install it first.")?;

        // Add candlekeep MCP server entry
        let servers = config
            .as_object_mut()
            .ok_or("Config is not an object")?
            .entry("mcpServers")
            .or_insert_with(|| serde_json::json!({}));

        if let Some(servers_obj) = servers.as_object_mut() {
            let home = dirs::home_dir()
                .map(|h| h.to_string_lossy().to_string())
                .unwrap_or_default();
            servers_obj.insert(
                "candlekeep".to_string(),
                serde_json::json!({
                    "command": ck_path.to_string_lossy(),
                    "args": ["mcp", "serve"],
                    "env": { "HOME": home }
                }),
            );
        }

        let output = serde_json::to_string_pretty(&config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        std::fs::write(&path, output)
            .map_err(|e| format!("Failed to write config: {}", e))?;

        Ok(())
    }
}

impl HostIntegration for CursorAdapter {
    fn kind(&self) -> HostKind {
        HostKind::Cursor
    }

    fn detect_host(&self, _platform: &PlatformInfo) -> bool {
        Self::is_cursor_installed()
    }

    fn detect_integration(&self) -> IntegrationStatus {
        let host_installed = Self::is_cursor_installed();
        let integration_installed = Self::is_mcp_configured();

        let status = if !host_installed {
            RequirementStatus::Missing
        } else if integration_installed {
            RequirementStatus::Satisfied
        } else {
            RequirementStatus::Missing
        };

        IntegrationStatus {
            host: HostKind::Cursor,
            host_installed,
            integration_installed,
            version: None,
            latest_version: None,
            install_method: "mcp-config".to_string(),
            status,
        }
    }

    fn uninstall(&self) -> ActionResult {
        let Some(path) = Self::mcp_config_path() else {
            return ActionResult::failure("Could not determine Cursor config path");
        };
        if !path.exists() {
            return ActionResult::success("Nothing to uninstall");
        }
        let Ok(content) = std::fs::read_to_string(&path) else {
            return ActionResult::failure("Failed to read MCP config");
        };
        let Ok(mut config) = serde_json::from_str::<serde_json::Value>(&content) else {
            return ActionResult::failure("Failed to parse MCP config");
        };
        if let Some(servers) = config.get_mut("mcpServers").and_then(|s| s.as_object_mut()) {
            servers.remove("candlekeep");
        }
        match serde_json::to_string_pretty(&config) {
            Ok(output) => {
                if let Err(e) = std::fs::write(&path, output) {
                    return ActionResult::failure(format!("Failed to write config: {}", e));
                }
                info!("CandleKeep MCP server removed from Cursor");
                let mut result = ActionResult::success("CandleKeep removed from Cursor");
                result.restart_required = true;
                result
            }
            Err(e) => ActionResult::failure(format!("Failed to serialize config: {}", e)),
        }
    }

    fn install(&self) -> ActionResult {
        if !Self::is_cursor_installed() {
            return ActionResult::failure("Cursor is not installed. Install Cursor first.");
        }

        match Self::write_mcp_config() {
            Ok(()) => {
                info!("CandleKeep MCP server configured for Cursor");
                let mut result = ActionResult::success("CandleKeep configured for Cursor");
                result.restart_required = true;
                result
            }
            Err(e) => ActionResult::failure(e),
        }
    }

    fn update(&self) -> ActionResult {
        // Re-write config with latest ck path
        self.install()
    }

    fn repair(&self) -> ActionResult {
        self.install()
    }

    fn requirements(&self, platform: &PlatformInfo) -> Vec<Requirement> {
        vec![
            Requirement {
                name: "Cursor".to_string(),
                description: "Install Cursor from cursor.com".to_string(),
                status: if Self::is_cursor_installed() {
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
