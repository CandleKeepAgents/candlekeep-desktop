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

    /// Path to the MCP config file.
    fn mcp_config_path() -> Option<PathBuf> {
        Self::config_dir().map(|d| d.join("mcp.json"))
    }

    fn is_mcp_configured() -> bool {
        let Some(path) = Self::mcp_config_path() else { return false };
        if !path.exists() { return false; }

        let Ok(content) = std::fs::read_to_string(&path) else { return false };
        let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) else { return false };

        json.get("mcpServers")
            .and_then(|s| s.get("candlekeep"))
            .is_some()
    }

    fn write_mcp_config() -> Result<(), String> {
        let path = Self::mcp_config_path()
            .ok_or("Could not determine Codex config path")?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config dir: {}", e))?;
        }

        let mut config: serde_json::Value = if path.exists() {
            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read MCP config: {}", e))?;
            serde_json::from_str(&content)
                .unwrap_or_else(|_| serde_json::json!({}))
        } else {
            serde_json::json!({})
        };

        let info = PlatformInfo::detect();
        let ck_path = paths::find_binary("ck", &info)
            .ok_or("CandleKeep CLI (ck) not found. Install it first.")?;

        let servers = config
            .as_object_mut()
            .ok_or("Config is not an object")?
            .entry("mcpServers")
            .or_insert_with(|| serde_json::json!({}));

        if let Some(servers_obj) = servers.as_object_mut() {
            servers_obj.insert(
                "candlekeep".to_string(),
                serde_json::json!({
                    "command": ck_path.to_string_lossy(),
                    "args": ["mcp", "serve"]
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

    fn install(&self) -> ActionResult {
        if !Self::is_codex_installed() {
            return ActionResult::failure("Codex is not installed. Install Codex first.");
        }

        match Self::write_mcp_config() {
            Ok(()) => {
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
