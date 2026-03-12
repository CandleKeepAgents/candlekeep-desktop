use std::path::PathBuf;
use std::process::Command;

/// Build a PATH that includes common macOS tool directories.
/// macOS GUI apps don't inherit the user's shell PATH, so we need this
/// for `which` fallbacks and spawning shell commands.
pub fn get_full_path() -> String {
    let home = dirs::home_dir().unwrap_or_default();
    let extra_paths = [
        "/opt/homebrew/bin",
        "/opt/homebrew/sbin",
        "/usr/local/bin",
        &format!("{}/.cargo/bin", home.display()),
        &format!("{}/.local/bin", home.display()),
    ];
    let system_path = std::env::var("PATH").unwrap_or_default();
    format!("{}:{}", extra_paths.join(":"), system_path)
}

/// Check if a binary exists at any of the given known paths.
fn exists_at_known_paths(paths: &[PathBuf]) -> bool {
    paths.iter().any(|p| p.exists())
}

#[tauri::command]
pub async fn check_homebrew() -> Result<bool, String> {
    let known = [
        PathBuf::from("/opt/homebrew/bin/brew"),
        PathBuf::from("/usr/local/bin/brew"),
    ];
    Ok(exists_at_known_paths(&known))
}

#[tauri::command]
pub async fn check_cargo() -> Result<bool, String> {
    let mut known = vec![
        PathBuf::from("/opt/homebrew/bin/cargo"),
        PathBuf::from("/usr/local/bin/cargo"),
    ];
    if let Some(home) = dirs::home_dir() {
        known.push(home.join(".cargo/bin/cargo"));
    }
    Ok(exists_at_known_paths(&known))
}

#[tauri::command]
pub async fn check_node() -> Result<bool, String> {
    let known = [
        PathBuf::from("/opt/homebrew/bin/node"),
        PathBuf::from("/usr/local/bin/node"),
    ];
    if exists_at_known_paths(&known) {
        return Ok(true);
    }
    // Fallback: node may be installed via nvm or other managers
    let output = Command::new("which")
        .arg("node")
        .env("PATH", get_full_path())
        .output()
        .map_err(|e| e.to_string())?;
    Ok(output.status.success())
}

#[tauri::command]
pub async fn check_xcode_clt() -> Result<bool, String> {
    let output = Command::new("xcode-select")
        .arg("-p")
        .output()
        .map_err(|e| e.to_string())?;
    Ok(output.status.success())
}

#[tauri::command]
pub async fn install_homebrew() -> Result<String, String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_full_path_contains_homebrew() {
        let path = get_full_path();
        assert!(path.contains("/opt/homebrew/bin"));
    }

    #[test]
    fn test_get_full_path_contains_cargo() {
        let path = get_full_path();
        assert!(path.contains(".cargo/bin"));
    }

    #[test]
    fn test_get_full_path_contains_local_bin() {
        let path = get_full_path();
        assert!(path.contains(".local/bin"));
    }

    #[test]
    fn test_get_full_path_contains_usr_local() {
        let path = get_full_path();
        assert!(path.contains("/usr/local/bin"));
    }
}
