use serde::Serialize;
use std::path::PathBuf;
use std::process::Command;
use tracing::{info, warn, error};

use super::system::get_full_path;

#[derive(Debug, Serialize)]
pub struct CliStatus {
    pub installed: bool,
    pub version: Option<String>,
    pub path: Option<String>,
    pub install_method: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AuthStatus {
    pub authenticated: bool,
    pub api_key_present: bool,
}

fn find_cli_path() -> Option<PathBuf> {
    let home = dirs::home_dir();

    // Check common locations (including cargo bin)
    let mut paths: Vec<PathBuf> = vec![
        PathBuf::from("/opt/homebrew/bin/ck"),
        PathBuf::from("/usr/local/bin/ck"),
    ];
    if let Some(ref h) = home {
        paths.push(h.join(".cargo/bin/ck"));
    }

    for p in &paths {
        if p.exists() {
            return Some(p.clone());
        }
    }

    // Fall back to which with expanded PATH (macOS GUI apps don't inherit shell PATH)
    if let Ok(output) = Command::new("which")
        .arg("ck")
        .env("PATH", get_full_path())
        .output()
    {
        if output.status.success() {
            let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path_str.is_empty() {
                return Some(PathBuf::from(path_str));
            }
        }
    }

    warn!("CK CLI binary not found in any known path");
    None
}

fn detect_install_method(path: &PathBuf) -> String {
    let path_str = path.to_string_lossy().to_lowercase();
    if path_str.contains("homebrew") || path_str.contains("cellar") {
        "homebrew".to_string()
    } else if path_str.contains(".cargo/bin") {
        "cargo".to_string()
    } else {
        "manual".to_string()
    }
}

#[tauri::command]
pub async fn check_cli_installed() -> Result<CliStatus, String> {
    match find_cli_path() {
        Some(path) => {
            let version = get_version_from_binary(&path);
            let install_method = detect_install_method(&path);
            Ok(CliStatus {
                installed: true,
                version,
                path: Some(path.to_string_lossy().to_string()),
                install_method: Some(install_method),
            })
        }
        None => Ok(CliStatus {
            installed: false,
            version: None,
            path: None,
            install_method: None,
        }),
    }
}

fn get_version_from_binary(path: &PathBuf) -> Option<String> {
    let output = Command::new(path)
        .arg("--version")
        .output()
        .ok()?;

    if output.status.success() {
        let version_str = String::from_utf8_lossy(&output.stdout);
        // Parse "ck X.Y.Z" or "candlekeep-cli X.Y.Z"
        let version = version_str
            .trim()
            .split_whitespace()
            .last()
            .map(|s| s.to_string());
        version
    } else {
        None
    }
}

#[tauri::command]
pub async fn get_cli_version() -> Result<Option<String>, String> {
    match find_cli_path() {
        Some(path) => Ok(get_version_from_binary(&path)),
        None => Ok(None),
    }
}

#[tauri::command]
pub async fn get_latest_cli_version() -> Result<Option<String>, String> {
    let url = "https://raw.githubusercontent.com/CandleKeepAgents/homebrew-candlekeep/main/Formula/candlekeep-cli.rb";

    let response = reqwest::get(url)
        .await
        .map_err(|e| format!("Failed to fetch formula: {}", e))?;

    if !response.status().is_success() {
        return Ok(None);
    }

    let body = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    // Parse version "X.Y.Z" from formula
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("version \"") {
            let version = trimmed
                .strip_prefix("version \"")
                .and_then(|s| s.strip_suffix('"'))
                .map(|s| s.to_string());
            return Ok(version);
        }
    }

    Ok(None)
}

#[tauri::command]
pub async fn install_cli() -> Result<String, String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg("brew install CandleKeepAgents/candlekeep/candlekeep-cli")
        .env("PATH", get_full_path())
        .output()
        .map_err(|e| {
            error!("Failed to start CLI installation: {}", e);
            format!("Failed to start CLI installation: {}", e)
        })?;

    if output.status.success() {
        info!("CLI installed successfully");
        Ok("CLI installed successfully".to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!("CLI installation failed: {}", stderr);
        Err(format!("CLI installation failed: {}", stderr))
    }
}

#[tauri::command]
pub async fn update_cli() -> Result<String, String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg("brew upgrade candlekeep-cli")
        .env("PATH", get_full_path())
        .output()
        .map_err(|e| {
            error!("Failed to start CLI update: {}", e);
            format!("Failed to start CLI update: {}", e)
        })?;

    if output.status.success() {
        info!("CLI updated successfully");
        Ok("CLI updated successfully".to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!("CLI update failed: {}", stderr);
        Err(format!("CLI update failed: {}", stderr))
    }
}

#[tauri::command]
pub async fn check_auth_status() -> Result<AuthStatus, String> {
    let config_path = dirs::home_dir()
        .ok_or("Could not find home directory")?
        .join(".candlekeep/config.toml");

    if !config_path.exists() {
        return Ok(AuthStatus {
            authenticated: false,
            api_key_present: false,
        });
    }

    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;

    let config: toml::Value = content
        .parse()
        .map_err(|e| format!("Failed to parse config: {}", e))?;

    let api_key_present = config
        .get("auth")
        .and_then(|auth| auth.get("api_key"))
        .and_then(|key| key.as_str())
        .map(|key| !key.is_empty())
        .unwrap_or(false);

    Ok(AuthStatus {
        authenticated: api_key_present,
        api_key_present,
    })
}

#[tauri::command]
pub async fn trigger_auth_login() -> Result<String, String> {
    // Find the ck binary first
    let ck_path = find_cli_path()
        .ok_or_else(|| "CandleKeep CLI not found. Please install it first.".to_string())?;

    let path_env = get_full_path();

    // Spawn ck auth login as a tokio task to properly manage its lifecycle
    tokio::spawn(async move {
        let result = tokio::process::Command::new(&ck_path)
            .args(["auth", "login"])
            .env("PATH", path_env)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .status()
            .await;

        match result {
            Ok(status) if status.success() => {
                info!("Auth login process completed successfully");
            }
            Ok(status) => {
                warn!("Auth login process exited with status: {}", status);
            }
            Err(e) => {
                error!("Auth login process failed: {}", e);
            }
        }
    });

    Ok("Auth login started — check your browser".to_string())
}

#[tauri::command]
pub async fn auth_logout() -> Result<String, String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg("ck auth logout")
        .env("PATH", get_full_path())
        .output()
        .map_err(|e| format!("Failed to logout: {}", e))?;

    if output.status.success() {
        Ok("Logged out successfully".to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Logout failed: {}", stderr))
    }
}
