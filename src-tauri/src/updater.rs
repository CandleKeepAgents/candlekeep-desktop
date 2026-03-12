use serde::Serialize;
use sha2::{Digest, Sha256};
use std::process::Command;
use tracing::{info, error};

#[derive(Debug, Serialize)]
pub struct AppUpdateInfo {
    pub update_available: bool,
    pub current_version: String,
    pub latest_version: Option<String>,
    pub download_url: Option<String>,
    pub dmg_url: Option<String>,
    pub checksum_url: Option<String>,
}

#[tauri::command]
pub async fn check_app_update() -> Result<AppUpdateInfo, String> {
    let current_version = env!("CARGO_PKG_VERSION").to_string();

    // Fetch latest release from GitHub
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
        info!("Update check returned non-success status: {}", response.status());
        return Ok(AppUpdateInfo {
            update_available: false,
            current_version,
            latest_version: None,
            download_url: None,
            dmg_url: None,
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

    // Find .dmg asset URL for macOS auto-update
    let dmg_url = release
        .get("assets")
        .and_then(|a| a.as_array())
        .and_then(|assets| {
            assets.iter().find_map(|asset| {
                let name = asset.get("name")?.as_str()?;
                if name.ends_with(".dmg") {
                    asset
                        .get("browser_download_url")
                        .and_then(|u| u.as_str())
                        .map(|s| s.to_string())
                } else {
                    None
                }
            })
        });

    // Find SHA256 checksums asset URL
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
        dmg_url,
        checksum_url,
    })
}

#[tauri::command]
pub async fn install_app_update(dmg_url: String, expected_checksum: Option<String>) -> Result<String, String> {
    let tmp_dir = std::env::temp_dir();
    let dmg_path = tmp_dir.join("CandleKeep-update.dmg");

    // Download the DMG
    let client = reqwest::Client::new();
    let response = client
        .get(&dmg_url)
        .header(
            "User-Agent",
            format!("candlekeep-desktop/{}", env!("CARGO_PKG_VERSION")),
        )
        .send()
        .await
        .map_err(|e| {
            error!("Failed to download update from {}: {}", dmg_url, e);
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

    std::fs::write(&dmg_path, &bytes)
        .map_err(|e| format!("Failed to save DMG: {}", e))?;

    // Open the DMG (mounts it and shows in Finder)
    Command::new("open")
        .arg(&dmg_path)
        .spawn()
        .map_err(|e| format!("Failed to open DMG: {}", e))?;

    Ok("Update downloaded and opened. Drag the new app to Applications to complete the update.".to_string())
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

    #[test]
    fn test_older_version_no_update() {
        let current = semver::Version::parse("0.2.0").unwrap();
        let latest = semver::Version::parse("0.1.0").unwrap();
        assert!(!(latest > current));
    }

    #[test]
    fn test_prerelease_version_comparison() {
        let current = semver::Version::parse("1.0.0-alpha").unwrap();
        let latest = semver::Version::parse("1.0.0").unwrap();
        assert!(latest > current);
    }

    #[test]
    fn test_invalid_semver_returns_false() {
        // Simulate the logic in check_app_update
        let result = if let (Ok(current), Ok(latest)) = (
            semver::Version::parse("0.1.0"),
            semver::Version::parse("not-a-version"),
        ) {
            latest > current
        } else {
            false
        };
        assert!(!result);
    }
}
