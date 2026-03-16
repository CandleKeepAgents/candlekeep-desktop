use std::path::PathBuf;
use std::time::Duration;
use tracing::{info, error};

use super::{Platform, PlatformInfo};

/// Result of an install/update/repair action.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ActionResult {
    pub ok: bool,
    pub message: String,
    pub details: Option<String>,
    pub restart_required: bool,
}

impl ActionResult {
    pub fn success(message: impl Into<String>) -> Self {
        Self { ok: true, message: message.into(), details: None, restart_required: false }
    }

    pub fn failure(message: impl Into<String>) -> Self {
        Self { ok: false, message: message.into(), details: None, restart_required: false }
    }

    #[allow(dead_code)]
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}

/// Install CK CLI from GitHub Releases.
/// On macOS, this is used when Homebrew is not available.
pub async fn install_cli_from_github(platform_info: &PlatformInfo) -> ActionResult {
    let target_triple = match (platform_info.platform, platform_info.arch.as_str()) {
        (Platform::Linux, "x86_64") => "x86_64-unknown-linux-gnu",
        (Platform::Linux, "aarch64") => "aarch64-unknown-linux-gnu",
        (Platform::Windows, "x86_64") => "x86_64-pc-windows-msvc",
        (Platform::Windows, "aarch64") => "aarch64-pc-windows-msvc",
        (Platform::MacOS, "x86_64") => "x86_64-apple-darwin",
        (Platform::MacOS, "aarch64") => "aarch64-apple-darwin",
        _ => {
            return ActionResult::failure(format!(
                "Unsupported platform/arch: {:?}/{}",
                platform_info.platform, platform_info.arch
            ));
        }
    };

    let archive_ext = match platform_info.platform {
        Platform::Windows => "zip",
        _ => "tar.gz",
    };

    // Fetch latest release from GitHub
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))
        .unwrap_or_else(|_| reqwest::Client::new());
    let releases_url = "https://api.github.com/repos/CandleKeepAgents/candlekeep-cli/releases";
    let response = match client
        .get(releases_url)
        .header("User-Agent", "candlekeep-desktop")
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to fetch releases: {}", e);
            return ActionResult::failure(format!("Failed to fetch releases: {}", e));
        }
    };

    if !response.status().is_success() {
        return ActionResult::failure(format!("GitHub API returned {}", response.status()));
    }

    let releases: Vec<serde_json::Value> = match response.json().await {
        Ok(r) => r,
        Err(e) => return ActionResult::failure(format!("Failed to parse releases: {}", e)),
    };

    // Find latest v* release
    let release = match releases.iter().find(|r| {
        r.get("tag_name")
            .and_then(|t| t.as_str())
            .map(|t| t.starts_with("v"))
            .unwrap_or(false)
    }) {
        Some(r) => r,
        None => return ActionResult::failure("No CLI release found"),
    };

    // Find matching asset
    let asset_name = format!("ck-{target_triple}.{archive_ext}");
    let download_url = match release
        .get("assets")
        .and_then(|a| a.as_array())
        .and_then(|assets| {
            assets.iter().find_map(|asset| {
                let name = asset.get("name")?.as_str()?;
                if name == asset_name {
                    asset.get("browser_download_url")?.as_str().map(|s| s.to_string())
                } else {
                    None
                }
            })
        })
    {
        Some(url) => url,
        None => {
            return ActionResult::failure(format!("No asset found matching {}", asset_name));
        }
    };

    // Download
    info!("Downloading CLI from {}", download_url);
    let response = match client
        .get(&download_url)
        .header("User-Agent", "candlekeep-desktop")
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => return ActionResult::failure(format!("Download failed: {}", e)),
    };

    let bytes = match response.bytes().await {
        Ok(b) => b,
        Err(e) => return ActionResult::failure(format!("Failed to read download: {}", e)),
    };

    // Extract to install dir
    let install_dir = &platform_info.paths.cli_install_dir;
    if let Err(e) = std::fs::create_dir_all(install_dir) {
        return ActionResult::failure(format!("Failed to create install dir: {}", e));
    }

    let binary_name = format!("ck{}", std::env::consts::EXE_SUFFIX);

    match platform_info.platform {
        Platform::Windows => {
            if let Err(e) = extract_zip(&bytes, install_dir, &binary_name) {
                return ActionResult::failure(format!("Failed to extract zip: {}", e));
            }
        }
        _ => {
            if let Err(e) = extract_tar_gz(&bytes, install_dir, &binary_name) {
                return ActionResult::failure(format!("Failed to extract tar.gz: {}", e));
            }
            // Set executable permission on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let bin_path = install_dir.join(&binary_name);
                if let Err(e) = std::fs::set_permissions(
                    &bin_path,
                    std::fs::Permissions::from_mode(0o755),
                ) {
                    return ActionResult::failure(format!("Failed to set permissions: {}", e));
                }
            }
        }
    }

    info!("CLI installed to {}", install_dir.display());
    ActionResult::success("CLI installed successfully")
}

fn extract_tar_gz(data: &[u8], dest: &PathBuf, binary_name: &str) -> Result<(), String> {
    use flate2::read::GzDecoder;
    use tar::Archive;

    let canonical_dest = dest.canonicalize().map_err(|e| {
        format!("Failed to canonicalize destination: {}", e)
    })?;

    let decoder = GzDecoder::new(data);
    let mut archive = Archive::new(decoder);

    for entry in archive.entries().map_err(|e| e.to_string())? {
        let mut entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path().map_err(|e| e.to_string())?;

        // Zip-slip prevention: reject entries with path traversal components
        if path.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
            return Err(format!("Archive contains path traversal: {}", path.display()));
        }

        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        if file_name == binary_name || file_name == "ck" {
            let dest_path = canonical_dest.join(binary_name);
            entry.unpack(&dest_path).map_err(|e| e.to_string())?;
            return Ok(());
        }
    }

    Err(format!("Binary '{}' not found in archive", binary_name))
}

fn extract_zip(data: &[u8], dest: &PathBuf, binary_name: &str) -> Result<(), String> {
    let canonical_dest = dest.canonicalize().map_err(|e| {
        format!("Failed to canonicalize destination: {}", e)
    })?;

    let cursor = std::io::Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor).map_err(|e| e.to_string())?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = file.name().to_string();
        let entry_path = std::path::Path::new(&name);

        // Zip-slip prevention: reject entries with path traversal components
        if entry_path.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
            return Err(format!("Archive contains path traversal: {}", name));
        }

        let file_name = entry_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        if file_name == binary_name || file_name == "ck.exe" || file_name == "ck" {
            let dest_path = canonical_dest.join(binary_name);
            let mut out = std::fs::File::create(&dest_path).map_err(|e| e.to_string())?;
            std::io::copy(&mut file, &mut out).map_err(|e| e.to_string())?;
            return Ok(());
        }
    }

    Err(format!("Binary '{}' not found in archive", binary_name))
}
