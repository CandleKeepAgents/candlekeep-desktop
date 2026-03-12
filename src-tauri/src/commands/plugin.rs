use serde::Serialize;
use std::path::PathBuf;
use std::process::Command;
use tracing::{info, warn, error};

#[derive(Debug, Serialize)]
pub struct PluginStatus {
    pub installed: bool,
    pub version: Option<String>,
    pub path: Option<String>,
}

fn get_plugin_path() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let plugin_path = home.join(".claude/plugins/marketplaces/candlekeep/plugins/candlekeep-cloud");
    if plugin_path.exists() {
        Some(plugin_path)
    } else {
        warn!("CandleKeep plugin path not found: {}", plugin_path.display());
        None
    }
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
    // Step 1: Add marketplace
    let output1 = Command::new("sh")
        .arg("-c")
        .arg("claude /plugin marketplace add CandleKeepAgents/candlekeep-marketplace")
        .output()
        .map_err(|e| format!("Failed to add marketplace: {}", e))?;

    if !output1.status.success() {
        let stderr = String::from_utf8_lossy(&output1.stderr);
        error!("Failed to add marketplace: {}", stderr);
        return Err(format!("Failed to add marketplace: {}", stderr));
    }

    // Step 2: Install plugin
    let output2 = Command::new("sh")
        .arg("-c")
        .arg("claude /plugin install candlekeep-cloud@candlekeep")
        .output()
        .map_err(|e| {
            error!("Failed to install plugin: {}", e);
            format!("Failed to install plugin: {}", e)
        })?;

    if output2.status.success() {
        info!("Plugin installed successfully");
        Ok("Plugin installed successfully".to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output2.stderr);
        error!("Plugin installation failed: {}", stderr);
        Err(format!("Plugin installation failed: {}", stderr))
    }
}

#[tauri::command]
pub async fn update_plugin() -> Result<String, String> {
    // Re-add marketplace to get latest
    let output = Command::new("sh")
        .arg("-c")
        .arg("claude /plugin marketplace add CandleKeepAgents/candlekeep-marketplace")
        .output()
        .map_err(|e| format!("Failed to update plugin: {}", e))?;

    if output.status.success() {
        info!("Plugin updated successfully");
        Ok("Plugin updated successfully".to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
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
