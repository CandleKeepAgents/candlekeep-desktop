use serde::Serialize;
use tracing::{info, debug, error};

use crate::platform::Platform;

#[derive(Debug, Serialize)]
pub struct AppUpdateInfo {
    pub update_available: bool,
    pub current_version: String,
    pub latest_version: Option<String>,
    pub download_url: Option<String>,
    pub asset_url: Option<String>,
    pub checksum_url: Option<String>,
}

/// Determine the expected asset suffix for the current platform.
fn platform_asset_suffix() -> &'static str {
    match Platform::current() {
        Platform::MacOS => ".dmg",
        Platform::Windows => ".msi",
        Platform::Linux => ".AppImage",
    }
}

#[tauri::command]
pub async fn check_app_update() -> Result<AppUpdateInfo, String> {
    let current_version = env!("CARGO_PKG_VERSION").to_string();

    let url = "https://api.github.com/repos/CandleKeepAgents/candlekeep-desktop/releases/latest";

    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header("User-Agent", format!("candlekeep-desktop/{}", current_version))
        .send()
        .await
        .map_err(|e| {
            error!("Failed to check for updates: {}", e);
            format!("Failed to check for updates: {}", e)
        })?;

    if !response.status().is_success() {
        debug!("Update check returned non-success status: {}", response.status());
        return Ok(AppUpdateInfo {
            update_available: false,
            current_version,
            latest_version: None,
            download_url: None,
            asset_url: None,
            checksum_url: None,
        });
    }

    let release: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse release: {}", e))?;

    let tag_name = release
        .get("tag_name")
        .and_then(|t| t.as_str())
        .map(|s| s.trim_start_matches('v').to_string());

    let download_url = release
        .get("html_url")
        .and_then(|u| u.as_str())
        .map(|s| s.to_string());

    let suffix = platform_asset_suffix();

    // Find platform-specific installer asset
    let asset_url = release
        .get("assets")
        .and_then(|a| a.as_array())
        .and_then(|assets| {
            assets.iter().find_map(|asset| {
                let name = asset.get("name")?.as_str()?;
                if name.ends_with(suffix) {
                    asset
                        .get("browser_download_url")
                        .and_then(|u| u.as_str())
                        .map(|s| s.to_string())
                } else {
                    None
                }
            })
        });

    let checksum_url = release
        .get("assets")
        .and_then(|a| a.as_array())
        .and_then(|assets| {
            assets.iter().find_map(|asset| {
                let name = asset.get("name")?.as_str()?;
                if name.contains("SHA256") {
                    asset
                        .get("browser_download_url")
                        .and_then(|u| u.as_str())
                        .map(|s| s.to_string())
                } else {
                    None
                }
            })
        });

    let update_available = if let Some(ref latest) = tag_name {
        if let (Ok(current), Ok(latest_ver)) = (
            semver::Version::parse(&current_version),
            semver::Version::parse(latest),
        ) {
            latest_ver > current
        } else {
            false
        }
    } else {
        false
    };

    info!(
        "Update check complete: current={}, latest={:?}, update_available={}",
        current_version, tag_name, update_available
    );

    Ok(AppUpdateInfo {
        update_available,
        current_version,
        latest_version: tag_name,
        download_url,
        asset_url,
        checksum_url,
    })
}

#[tauri::command]
pub async fn install_app_update(asset_url: String, expected_checksum: Option<String>) -> Result<String, String> {
    let tmp_dir = std::env::temp_dir();
    let suffix = platform_asset_suffix();
    let file_name = format!("CandleKeep-update{}", suffix);
    let file_path = tmp_dir.join(&file_name);

    let client = reqwest::Client::new();
    let response = client
        .get(&asset_url)
        .header(
            "User-Agent",
            format!("candlekeep-desktop/{}", env!("CARGO_PKG_VERSION")),
        )
        .send()
        .await
        .map_err(|e| {
            error!("Failed to download update from {}: {}", asset_url, e);
            format!("Failed to download update: {}", e)
        })?;

    if !response.status().is_success() {
        error!("Update download failed with status: {}", response.status());
        return Err(format!(
            "Download failed with status: {}",
            response.status()
        ));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read download: {}", e))?;

    // Verify checksum if provided
    if let Some(ref expected) = expected_checksum {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let computed = format!("{:x}", hasher.finalize());
        if computed != *expected {
            return Err(format!(
                "Checksum mismatch: expected {}, got {}",
                expected, computed
            ));
        }
    }

    std::fs::write(&file_path, &bytes)
        .map_err(|e| format!("Failed to save update: {}", e))?;

    // Open the downloaded installer
    match Platform::current() {
        Platform::MacOS => {
            // Open DMG (mounts and shows in Finder)
            std::process::Command::new("open")
                .arg(&file_path)
                .spawn()
                .map_err(|e| format!("Failed to open DMG: {}", e))?;
            Ok("Update downloaded and opened. Drag the new app to Applications to complete the update.".to_string())
        }
        Platform::Windows => {
            // Run MSI/NSIS installer
            std::process::Command::new("cmd")
                .args(["/C", "start", "", &file_path.to_string_lossy()])
                .spawn()
                .map_err(|e| format!("Failed to run installer: {}", e))?;
            Ok("Update downloaded. The installer will guide you through the update.".to_string())
        }
        Platform::Linux => {
            // Make AppImage executable and notify user
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&file_path, std::fs::Permissions::from_mode(0o755))
                    .map_err(|e| format!("Failed to set permissions: {}", e))?;
            }
            Ok(format!("Update downloaded to {}. Replace the current AppImage to complete the update.", file_path.display()))
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_newer_version_detected() {
        let current = semver::Version::parse("0.1.0").unwrap();
        let latest = semver::Version::parse("0.2.0").unwrap();
        assert!(latest > current);
    }

    #[test]
    fn test_same_version_no_update() {
        let current = semver::Version::parse("0.1.0").unwrap();
        let latest = semver::Version::parse("0.1.0").unwrap();
        assert!(!(latest > current));
    }
}
