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
    // Native auth flow: bind a TCP listener, open browser, wait for callback.
    // This avoids spawning `ck auth login` which panics (exit 101) when run
    // from a macOS GUI app due to stdin/stdout handling issues.

    let listener = std::net::TcpListener::bind("127.0.0.1:0")
        .map_err(|e| format!("Failed to start local auth server: {}", e))?;
    let port = listener.local_addr()
        .map_err(|e| format!("Failed to get listener port: {}", e))?
        .port();

    // Read API base URL from config (same logic as CLI)
    let api_url = get_api_base_url();
    let auth_url = format!("{}/cli-auth?port={}", api_url, port);

    info!("Opening browser for auth: {}", auth_url);

    // Open browser from the Tauri process (works reliably in GUI context)
    if let Err(e) = open::that(&auth_url) {
        warn!("Failed to open browser: {}", e);
        return Err(format!("Failed to open browser: {}. Visit {} manually.", e, auth_url));
    }

    // Wait for the OAuth callback in a background task
    tokio::spawn(async move {
        listener.set_nonblocking(false).ok();
        let handle = std::thread::spawn(move || -> Result<String, String> {
            use std::io::{BufRead, BufReader, Write};
            // Block until the browser sends the callback (or the listener is dropped)
            let (mut stream, _) = listener.accept()
                .map_err(|e| format!("Failed to accept callback: {}", e))?;

            let mut reader = BufReader::new(&stream);
            let mut request_line = String::new();
            reader.read_line(&mut request_line)
                .map_err(|e| format!("Failed to read callback: {}", e))?;

            // Parse: GET /callback?key=ck_xxx HTTP/1.1
            let api_key = request_line
                .split_whitespace()
                .nth(1)
                .and_then(|path| path.strip_prefix("/callback?key="))
                .map(|s| s.to_string())
                .ok_or_else(|| "Invalid callback URL".to_string())?;

            // Send success response to browser
            let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nConnection: close\r\n\r\n\
                <!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>CandleKeep</title>\
                <style>body{font-family:-apple-system,sans-serif;display:flex;justify-content:center;align-items:center;height:100vh;margin:0;background:#1a1a1a;color:#fff}\
                .c{text-align:center}.s{color:#22c55e;font-size:3rem;margin-bottom:1rem}</style></head>\
                <body><div class=\"c\"><div class=\"s\">&#x2713;</div><h1>Authentication Successful</h1>\
                <p style=\"color:#888\">You can close this window.</p></div></body></html>";
            stream.write_all(response.as_bytes()).ok();
            stream.flush().ok();

            Ok(api_key)
        });

        match handle.join() {
            Ok(Ok(api_key)) => {
                if let Err(e) = save_api_key_to_config(&api_key) {
                    error!("Failed to save API key: {}", e);
                } else {
                    info!("Auth login completed successfully (native flow)");
                }
            }
            Ok(Err(e)) => {
                error!("Auth callback failed: {}", e);
            }
            Err(_) => {
                error!("Auth callback handler panicked");
            }
        }
    });

    Ok("Auth login started — check your browser".to_string())
}

/// Read the CandleKeep API base URL from config or use default.
fn get_api_base_url() -> String {
    if let Ok(url) = std::env::var("CANDLEKEEP_API_URL") {
        return url;
    }
    let config_path = dirs::home_dir()
        .map(|h| h.join(".candlekeep/config.toml"));
    if let Some(path) = config_path {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(config) = content.parse::<toml::Value>() {
                if let Some(url) = config.get("api").and_then(|a| a.get("url")).and_then(|u| u.as_str()) {
                    return url.to_string();
                }
            }
        }
    }
    "https://www.getcandlekeep.com".to_string()
}

/// Save API key to ~/.candlekeep/config.toml (same format as CLI).
fn save_api_key_to_config(api_key: &str) -> Result<(), String> {
    let home = dirs::home_dir().ok_or("Could not find home directory")?;
    let config_dir = home.join(".candlekeep");
    let config_path = config_dir.join("config.toml");

    // Ensure directory exists
    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir)
            .map_err(|e| format!("Failed to create config dir: {}", e))?;
    }

    // Load existing config or create default
    let mut config: toml::Value = if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config: {}", e))?;
        content.parse().unwrap_or_else(|_| toml::Value::Table(toml::map::Map::new()))
    } else {
        toml::Value::Table(toml::map::Map::new())
    };

    // Set auth.api_key
    let table = config.as_table_mut().ok_or("Config is not a table")?;
    let auth = table.entry("auth").or_insert_with(|| toml::Value::Table(toml::map::Map::new()));
    if let Some(auth_table) = auth.as_table_mut() {
        auth_table.insert("api_key".to_string(), toml::Value::String(api_key.to_string()));
    }

    let content = toml::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    std::fs::write(&config_path, content)
        .map_err(|e| format!("Failed to write config: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn auth_logout() -> Result<String, String> {
    // Clear API key directly from config file (same format as CLI).
    // This avoids spawning a CLI subprocess which can have PATH issues in GUI apps.
    let home = dirs::home_dir().ok_or("Could not find home directory")?;
    let config_path = home.join(".candlekeep/config.toml");

    if !config_path.exists() {
        info!("No config file, already logged out");
        return Ok("Logged out successfully".to_string());
    }

    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;
    let mut config: toml::Value = content.parse()
        .map_err(|e| format!("Failed to parse config: {}", e))?;

    // Remove auth.api_key
    if let Some(table) = config.as_table_mut() {
        if let Some(auth) = table.get_mut("auth").and_then(|a| a.as_table_mut()) {
            auth.remove("api_key");
        }
    }

    let updated = toml::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    std::fs::write(&config_path, updated)
        .map_err(|e| format!("Failed to write config: {}", e))?;

    info!("User logged out successfully");
    Ok("Logged out successfully".to_string())
}
