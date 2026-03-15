use std::path::PathBuf;
use std::process::Command;

use super::PlatformInfo;

/// Scan ~/.nvm/versions/node/*/bin for the latest version directory.
pub fn find_latest_nvm_bin(home: &std::path::Path) -> Option<PathBuf> {
    let nvm_dir = home.join(".nvm/versions/node");
    if !nvm_dir.exists() {
        return None;
    }
    let mut versions: Vec<PathBuf> = std::fs::read_dir(&nvm_dir)
        .ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    versions.sort();
    versions.last().map(|v| v.join("bin"))
}

/// Build a PATH string that includes platform-specific binary directories.
/// GUI apps (especially on macOS) do not inherit the user's shell PATH.
pub fn get_full_path(info: &PlatformInfo) -> String {
    let mut paths: Vec<String> = info
        .paths
        .extra_bin_dirs
        .iter()
        .map(|p| p.display().to_string())
        .collect();

    if let Ok(sys) = std::env::var("PATH") {
        paths.push(sys);
    }

    let sep = info.paths.path_separator;
    paths.join(&sep.to_string())
}

/// Find a binary by name, searching platform-specific known paths first,
/// then falling back to `which` / `where.exe` with an expanded PATH.
pub fn find_binary(name: &str, info: &PlatformInfo) -> Option<PathBuf> {
    let exe = format!("{}{}", name, std::env::consts::EXE_SUFFIX);

    // 1. Check platform-specific known dirs
    for dir in &info.paths.extra_bin_dirs {
        let candidate = dir.join(&exe);
        if candidate.exists() {
            return Some(candidate);
        }
    }

    // 2. Cross-platform common paths
    if let Some(home) = dirs::home_dir() {
        for dir in [home.join(".cargo/bin"), home.join(".local/bin")] {
            let candidate = dir.join(&exe);
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    // 3. Windows: check .cmd wrappers (npm global installs)
    #[cfg(target_os = "windows")]
    {
        for dir in &info.paths.extra_bin_dirs {
            let candidate = dir.join(format!("{name}.cmd"));
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    // 4. Fallback: which/where with expanded PATH
    let which_cmd = if cfg!(windows) { "where.exe" } else { "which" };
    Command::new(which_cmd)
        .arg(name)
        .env("PATH", get_full_path(info))
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| {
            let p = PathBuf::from(String::from_utf8_lossy(&o.stdout).lines().next()?.trim());
            p.exists().then_some(p)
        })
}

/// Check if a binary exists at any of the given known paths.
pub fn exists_at_known_paths(paths: &[PathBuf]) -> bool {
    paths.iter().any(|p| p.exists())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::PlatformInfo;

    #[test]
    fn test_get_full_path_contains_common_dirs() {
        let info = PlatformInfo::detect();
        let path = get_full_path(&info);
        assert!(path.contains(".cargo/bin"));
        assert!(path.contains(".local/bin"));
    }

    #[test]
    fn test_find_binary_finds_sh() {
        // sh should exist on all Unix-like systems
        #[cfg(not(target_os = "windows"))]
        {
            let info = PlatformInfo::detect();
            let result = find_binary("sh", &info);
            assert!(result.is_some());
        }
    }
}
