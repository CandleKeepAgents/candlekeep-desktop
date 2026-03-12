use serde::Serialize;
use std::path::PathBuf;
use std::process::Command;
use tracing::{info, warn, error, debug};

use super::system::get_full_path;

#[derive(Debug, Serialize)]
pub struct PluginStatus {
    pub installed: bool,
    pub version: Option<String>,
    pub path: Option<String>,
}

fn get_plugin_path() -> Option<PathBuf> {
    let home = dirs::home_dir()?;

    // Primary: read install path from installed_plugins.json (source of truth)
    let installed_json_path = home.join(".claude/plugins/installed_plugins.json");
    if let Ok(content) = std::fs::read_to_string(&installed_json_path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(entries) = json.get("plugins")
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

    // Fallback: check known paths (cache layout and legacy marketplace layout)
    let candidates = [
        home.join(".claude/plugins/cache/candlekeep/candlekeep-cloud"),
        home.join(".claude/plugins/marketplaces/candlekeep/plugins/candlekeep-cloud"),
    ];

    for base in &candidates {
        if base.exists() {
            // For cache layout, find the latest version subdirectory
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
            // If the base itself has plugin.json, use it directly (legacy layout)
            if base.join(".claude-plugin/plugin.json").exists() {
                debug!("Found plugin at base path: {}", base.display());
                return Some(base.clone());
            }
        }
    }

    warn!("CandleKeep plugin not found in installed_plugins.json or known paths");
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

#[tauri::command]
pub async fn check_plugin_installed() -> Result<PluginStatus, String> {
    match get_plugin_path() {
        Some(path) => {
            let version = read_plugin_version(&path);
            Ok(PluginStatus {
                installed: true,
                version,
                path: Some(path.to_string_lossy().to_string()),
            })
        }
        None => Ok(PluginStatus {
            installed: false,
            version: None,
            path: None,
        }),
    }
}

#[tauri::command]
pub async fn get_plugin_version() -> Result<Option<String>, String> {
    match get_plugin_path() {
        Some(path) => Ok(read_plugin_version(&path)),
        None => Ok(None),
    }
}

#[tauri::command]
pub async fn install_plugin() -> Result<String, String> {
    let path_env = get_full_path();
    debug!("install_plugin using PATH: {}", path_env);

    // Step 1: Add marketplace
    let output1 = Command::new("sh")
        .arg("-c")
        .arg("claude /plugin marketplace add CandleKeepAgents/candlekeep-marketplace")
        .env("PATH", &path_env)
        .output()
        .map_err(|e| format!("Failed to add marketplace: {}", e))?;

    let stdout1 = String::from_utf8_lossy(&output1.stdout);
    let stderr1 = String::from_utf8_lossy(&output1.stderr);
    debug!("Add marketplace stdout: {}", stdout1);
    debug!("Add marketplace stderr: {}", stderr1);

    if !output1.status.success() {
        error!("Failed to add marketplace: {}", stderr1);
        return Err(format!("Failed to add marketplace: {}", stderr1));
    }

    // Step 2: Install plugin
    let output2 = Command::new("sh")
        .arg("-c")
        .arg("claude /plugin install candlekeep-cloud@candlekeep")
        .env("PATH", &path_env)
        .output()
        .map_err(|e| {
            error!("Failed to install plugin: {}", e);
            format!("Failed to install plugin: {}", e)
        })?;

    let stdout2 = String::from_utf8_lossy(&output2.stdout);
    let stderr2 = String::from_utf8_lossy(&output2.stderr);
    debug!("Install plugin stdout: {}", stdout2);
    debug!("Install plugin stderr: {}", stderr2);

    if !output2.status.success() {
        error!("Plugin installation failed: {}", stderr2);
        return Err(format!("Plugin installation failed: {}", stderr2));
    }

    // Verify the plugin path actually exists after install
    match get_plugin_path() {
        Some(path) => {
            info!("Plugin installed successfully at: {}", path.display());
            Ok("Plugin installed successfully".to_string())
        }
        None => {
            error!("Plugin install command succeeded but plugin path not found");
            Err("Plugin install appeared to succeed but plugin files were not found. Check that Claude Code is installed and accessible.".to_string())
        }
    }
}

#[tauri::command]
pub async fn update_plugin() -> Result<String, String> {
    let path_env = get_full_path();
    debug!("update_plugin using PATH: {}", path_env);

    // Re-add marketplace to get latest
    let output = Command::new("sh")
        .arg("-c")
        .arg("claude /plugin marketplace add CandleKeepAgents/candlekeep-marketplace")
        .env("PATH", &path_env)
        .output()
        .map_err(|e| format!("Failed to update plugin: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    debug!("Update plugin stdout: {}", stdout);
    debug!("Update plugin stderr: {}", stderr);

    if output.status.success() {
        info!("Plugin updated successfully");
        Ok("Plugin updated successfully".to_string())
    } else {
        error!("Plugin update failed: {}", stderr);
        Err(format!("Plugin update failed: {}", stderr))
    }
}

#[tauri::command]
pub async fn check_claude_code_installed() -> Result<bool, String> {
    // Check known paths directly (macOS GUI apps don't inherit shell PATH)
    let known_paths = [
        PathBuf::from("/opt/homebrew/bin/claude"),
        PathBuf::from("/usr/local/bin/claude"),
    ];
    if let Some(home) = dirs::home_dir() {
        let local_bin = home.join(".local/bin/claude");
        if local_bin.exists() {
            return Ok(true);
        }
    }
    for p in &known_paths {
        if p.exists() {
            return Ok(true);
        }
    }
    Ok(false)
}
