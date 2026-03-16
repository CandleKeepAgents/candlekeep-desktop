use std::path::PathBuf;
use std::process::Command;

use crate::platform::{self, PlatformInfo};
use crate::platform::paths;

/// Build a PATH that includes platform-specific tool directories.
/// Delegates to the platform layer. Used by tests.
#[cfg(test)]
pub fn get_full_path() -> String {
    let info = PlatformInfo::detect();
    paths::get_full_path(&info)
}

/// Check if a binary exists at any of the given known paths.
fn exists_at_known_paths(known: &[PathBuf]) -> bool {
    paths::exists_at_known_paths(known)
}

#[tauri::command]
pub async fn check_homebrew() -> Result<bool, String> {
    // Homebrew is macOS/Linux only
    match platform::Platform::current() {
        platform::Platform::Windows => Ok(false),
        _ => {
            let known = [
                PathBuf::from("/opt/homebrew/bin/brew"),
                PathBuf::from("/usr/local/bin/brew"),
            ];
            Ok(exists_at_known_paths(&known))
        }
    }
}

#[tauri::command]
pub async fn check_cargo() -> Result<bool, String> {
    let info = PlatformInfo::detect();
    Ok(paths::find_binary("cargo", &info).is_some())
}

#[tauri::command]
pub async fn check_node() -> Result<bool, String> {
    let info = PlatformInfo::detect();
    Ok(paths::find_binary("node", &info).is_some())
}

#[tauri::command]
pub async fn check_xcode_clt() -> Result<bool, String> {
    // Xcode CLT is macOS only
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("xcode-select")
            .arg("-p")
            .output()
            .map_err(|e| e.to_string())?;
        Ok(output.status.success())
    }
    #[cfg(not(target_os = "macos"))]
    {
        Ok(true) // Not required on non-macOS
    }
}

#[tauri::command]
pub async fn install_homebrew() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        return Err("Homebrew is not available on Windows".to_string());
    }
    #[cfg(not(target_os = "windows"))]
    {
        let output = Command::new("sh")
            .arg("-c")
            .arg("NONINTERACTIVE=1 /bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\"")
            .output()
            .map_err(|e| format!("Failed to start Homebrew installation: {}", e))?;

        if output.status.success() {
            Ok("Homebrew installed successfully".to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("Homebrew installation failed: {}", stderr))
        }
    }
}

#[tauri::command]
pub async fn get_platform_info() -> Result<PlatformInfo, String> {
    Ok(PlatformInfo::detect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_full_path_contains_common_dirs() {
        let path = get_full_path();
        assert!(path.contains(".cargo/bin"));
        assert!(path.contains(".local/bin"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_get_full_path_contains_homebrew() {
        let path = get_full_path();
        assert!(path.contains("/opt/homebrew/bin"));
    }
}
