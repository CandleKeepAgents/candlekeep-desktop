use std::path::PathBuf;
use tracing::info;

use crate::platform::installer::ActionResult;
use crate::platform::paths;
use crate::platform::PlatformInfo;

use super::{HostIntegration, HostKind, IntegrationStatus, Requirement, RequirementStatus};

pub struct OpenCodeAdapter;

impl OpenCodeAdapter {
    pub fn new() -> Self {
        Self
    }

    /// Get the OpenCode config directory.
    /// OpenCode v1.2+ uses ~/.config/opencode/opencode.json for global config.
    fn config_dir() -> Option<PathBuf> {
        let home = dirs::home_dir()?;
        Some(home.join(".config").join("opencode"))
    }

    fn is_opencode_installed() -> bool {
        if let Some(home) = dirs::home_dir() {
            if home.join(".opencode/bin/opencode").exists() {
                return true;
            }
            if home.join("go/bin/opencode").exists() {
                return true;
            }
        }
        let info = PlatformInfo::detect();
        paths::find_binary("opencode", &info).is_some()
    }

    /// OpenCode v1.2+ config file is opencode.json (no leading dot).
    fn config_path() -> Option<PathBuf> {
        Self::config_dir().map(|d| d.join("opencode.json"))
    }

    fn is_mcp_configured() -> bool {
        let Some(path) = Self::config_path() else { return false };
        if !path.exists() { return false; }

        let Ok(content) = std::fs::read_to_string(&path) else { return false };
        let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) else { return false };

        // OpenCode v1.2+ uses "mcp" key (not "mcpServers")
        json.get("mcp")
            .and_then(|s| s.get("candlekeep"))
            .is_some()
    }

    fn write_mcp_config() -> Result<(), String> {
        let path = Self::config_path()
            .ok_or("Could not determine OpenCode config path")?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config dir: {}", e))?;
        }

        let mut config: serde_json::Value = if path.exists() {
            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read opencode.json: {}", e))?;
            serde_json::from_str(&content)
                .unwrap_or_else(|_| serde_json::json!({}))
        } else {
            serde_json::json!({})
        };

        let info = PlatformInfo::detect();
        let ck_path = paths::find_binary("ck", &info)
            .ok_or("CandleKeep CLI (ck) not found. Install it first.")?;

        // OpenCode v1.2+ format: type "local", command is an array [binary, ...args]
        let server_entry = serde_json::json!({
            "type": "local",
            "command": [ck_path.to_string_lossy(), "mcp", "serve"]
        });

        let servers = config
            .as_object_mut()
            .ok_or("Config is not an object")?
            .entry("mcp")
            .or_insert_with(|| serde_json::json!({}));

        if let Some(servers_obj) = servers.as_object_mut() {
            servers_obj.insert("candlekeep".to_string(), server_entry);
        }

        let output = serde_json::to_string_pretty(&config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        std::fs::write(&path, output)
            .map_err(|e| format!("Failed to write config: {}", e))?;

        Ok(())
    }
}

impl HostIntegration for OpenCodeAdapter {
    fn kind(&self) -> HostKind {
        HostKind::OpenCode
    }

    fn detect_host(&self, _platform: &PlatformInfo) -> bool {
        Self::is_opencode_installed()
    }

    fn detect_integration(&self) -> IntegrationStatus {
        let host_installed = Self::is_opencode_installed();
        let integration_installed = Self::is_mcp_configured();

        let status = if !host_installed {
            RequirementStatus::Missing
        } else if integration_installed {
            RequirementStatus::Satisfied
        } else {
            RequirementStatus::Missing
        };

        IntegrationStatus {
            host: HostKind::OpenCode,
            host_installed,
            integration_installed,
            version: None,
            latest_version: None,
            install_method: "mcp-config".to_string(),
            status,
        }
    }

    fn uninstall(&self) -> ActionResult {
        let Some(path) = Self::config_path() else {
            return ActionResult::failure("Could not determine OpenCode config path");
        };
        if !path.exists() {
            return ActionResult::success("Nothing to uninstall");
        }
        let Ok(content) = std::fs::read_to_string(&path) else {
            return ActionResult::failure("Failed to read opencode.json");
        };
        let Ok(mut config) = serde_json::from_str::<serde_json::Value>(&content) else {
            return ActionResult::failure("Failed to parse opencode.json");
        };
        if let Some(servers) = config.get_mut("mcp").and_then(|s| s.as_object_mut()) {
            servers.remove("candlekeep");
        }
        match serde_json::to_string_pretty(&config) {
            Ok(output) => {
                if let Err(e) = std::fs::write(&path, output) {
                    return ActionResult::failure(format!("Failed to write config: {}", e));
                }
                info!("CandleKeep MCP server removed from OpenCode");
                let mut result = ActionResult::success("CandleKeep removed from OpenCode");
                result.restart_required = true;
                result
            }
            Err(e) => ActionResult::failure(format!("Failed to serialize config: {}", e)),
        }
    }

    fn install(&self) -> ActionResult {
        if !Self::is_opencode_installed() {
            return ActionResult::failure("OpenCode is not installed. Install OpenCode first.");
        }

        match Self::write_mcp_config() {
            Ok(()) => {
                info!("CandleKeep MCP server configured for OpenCode");
                let mut result = ActionResult::success("CandleKeep configured for OpenCode");
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
                name: "OpenCode".to_string(),
                description: "Install OpenCode CLI".to_string(),
                status: if Self::is_opencode_installed() {
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
