pub mod installer;
pub mod paths;
pub mod shell;
pub mod tray;

use serde::Serialize;
use std::path::PathBuf;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    MacOS,
    Windows,
    Linux,
}

impl Platform {
    pub fn current() -> Self {
        #[cfg(target_os = "macos")]
        { Platform::MacOS }
        #[cfg(target_os = "windows")]
        { Platform::Windows }
        #[cfg(target_os = "linux")]
        { Platform::Linux }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PlatformPaths {
    pub cli_install_dir: PathBuf,
    pub config_dir: PathBuf,
    pub extra_bin_dirs: Vec<PathBuf>,
    pub path_separator: char,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlatformInfo {
    pub platform: Platform,
    pub arch: String,
    pub tray_supported: bool,
    pub paths: PlatformPaths,
}

impl PlatformInfo {
    pub fn detect() -> Self {
        let platform = Platform::current();
        let arch = std::env::consts::ARCH.to_string();
        let home = dirs::home_dir().unwrap_or_default();

        let paths = match platform {
            Platform::MacOS => PlatformPaths {
                cli_install_dir: home.join(".local/bin"),
                config_dir: home.join(".candlekeep"),
                extra_bin_dirs: vec![
                    PathBuf::from("/opt/homebrew/bin"),
                    PathBuf::from("/opt/homebrew/sbin"),
                    PathBuf::from("/usr/local/bin"),
                    home.join(".cargo/bin"),
                    home.join(".local/bin"),
                ],
                path_separator: ':',
            },
            Platform::Linux => {
                let mut extra = vec![
                    PathBuf::from("/usr/local/bin"),
                    PathBuf::from("/usr/bin"),
                    PathBuf::from("/snap/bin"),
                    home.join(".cargo/bin"),
                    home.join(".local/bin"),
                    home.join(".volta/bin"),
                ];
                // Scan for nvm node versions
                if let Some(nvm_bin) = paths::find_latest_nvm_bin(&home) {
                    extra.push(nvm_bin);
                }
                PlatformPaths {
                    cli_install_dir: home.join(".local/bin"),
                    config_dir: home.join(".candlekeep"),
                    extra_bin_dirs: extra,
                    path_separator: ':',
                }
            }
            Platform::Windows => {
                let mut extra = vec![
                    home.join(".cargo/bin"),
                    home.join(".local/bin"),
                    home.join("scoop/shims"),
                    home.join(".volta/bin"),
                ];
                if let Ok(appdata) = std::env::var("APPDATA") {
                    extra.push(PathBuf::from(format!("{appdata}\\npm")));
                }
                if let Ok(pf) = std::env::var("ProgramFiles") {
                    extra.push(PathBuf::from(format!("{pf}\\nodejs")));
                }
                let cli_install_dir = std::env::var("LOCALAPPDATA")
                    .map(|la| PathBuf::from(format!("{la}\\Programs\\candlekeep")))
                    .unwrap_or_else(|_| home.join(".local/bin"));
                PlatformPaths {
                    cli_install_dir,
                    config_dir: home.join(".candlekeep"),
                    extra_bin_dirs: extra,
                    path_separator: ';',
                }
            }
        };

        PlatformInfo {
            platform,
            arch,
            tray_supported: true, // assume true; Linux fallback handled at runtime
            paths,
        }
    }
}
