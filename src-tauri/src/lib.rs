mod commands;
mod integrations;
mod platform;
mod state;
mod updater;

use tauri::Manager;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use tracing_appender::rolling;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let log_dir = dirs::home_dir()
        .map(|h| h.join(".candlekeep/logs"))
        .unwrap_or_else(|| std::env::temp_dir());
    let file_appender = rolling::daily(&log_dir, "candlekeep-desktop.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(fmt::layer().with_writer(std::io::stdout))
        .with(fmt::layer().with_writer(non_blocking).with_ansi(false))
        .init();

    // Install a panic hook that writes crash info to a log file
    let crash_log_dir = log_dir.clone();
    std::panic::set_hook(Box::new(move |info| {
        let msg = format!("PANIC: {}", info);
        eprintln!("{}", msg);
        let crash_path = crash_log_dir.join("crash.log");
        let _ = std::fs::write(&crash_path, &msg);
    }));

    let platform_info = platform::PlatformInfo::detect();
    tracing::info!("CandleKeep Desktop starting");
    tracing::info!(
        "Platform: {:?}, Arch: {}, Log dir: {}",
        platform_info.platform,
        platform_info.arch,
        log_dir.display()
    );

    #[tauri::command]
    fn mark_setup_complete(app_handle: tauri::AppHandle) -> Result<(), String> {
        let mut app_state = state::AppState::load();
        app_state.setup_completed = true;
        app_state.save()?;

        // Now that setup is done, hide from dock
        #[cfg(target_os = "macos")]
        let _ = app_handle.set_activation_policy(tauri::ActivationPolicy::Accessory);

        #[cfg(not(target_os = "macos"))]
        let _ = &app_handle;

        Ok(())
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            platform::tray::setup_tray(app)?;

            let app_state = state::AppState::load();
            if !app_state.setup_completed {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // State commands
            mark_setup_complete,
            // System / platform commands
            commands::system::check_homebrew,
            commands::system::check_cargo,
            commands::system::check_node,
            commands::system::check_xcode_clt,
            commands::system::install_homebrew,
            commands::system::get_platform_info,
            // CLI commands
            commands::cli::check_cli_installed,
            commands::cli::get_cli_version,
            commands::cli::get_latest_cli_version,
            commands::cli::install_cli,
            commands::cli::update_cli,
            commands::cli::check_auth_status,
            commands::cli::trigger_auth_login,
            commands::cli::auth_logout,
            // Legacy plugin commands (kept for backward compatibility)
            commands::plugin::check_plugin_installed,
            commands::plugin::get_plugin_version,
            commands::plugin::install_plugin,
            commands::plugin::update_plugin,
            commands::plugin::check_claude_code_installed,
            // Generic integration commands
            integrations::list_integrations,
            integrations::check_integration,
            integrations::install_integration,
            integrations::uninstall_integration,
            integrations::update_integration,
            integrations::repair_integration,
            // Metrics commands
            commands::metrics::fetch_whoami,
            commands::metrics::fetch_metrics,
            // Updater commands
            updater::check_app_update,
            updater::install_app_update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
