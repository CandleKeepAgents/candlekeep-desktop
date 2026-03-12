mod commands;
mod integrations;
mod state;
mod updater;

use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    Manager,
};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use tracing_appender::rolling;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let log_dir = dirs::home_dir()
        .map(|h| h.join(".candlekeep/logs"))
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
    let file_appender = rolling::daily(&log_dir, "candlekeep-desktop.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(fmt::layer().with_writer(std::io::stdout))
        .with(fmt::layer().with_writer(non_blocking).with_ansi(false))
        .init();

    tracing::info!("CandleKeep Desktop starting");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Build tray menu
            let show_item = MenuItemBuilder::with_id("show", "Show CandleKeep").build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let menu = MenuBuilder::new(app)
                .item(&show_item)
                .separator()
                .item(&quit_item)
                .build()?;

            // Create tray icon
            let mut tray_builder = TrayIconBuilder::new();
            if let Some(icon) = app.default_window_icon() {
                tray_builder = tray_builder.icon(icon.clone());
            }
            tray_builder
                .menu(&menu)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            // Hide from dock on macOS
            #[cfg(target_os = "macos")]
            {
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // System commands
            commands::system::check_homebrew,
            commands::system::check_cargo,
            commands::system::check_node,
            commands::system::check_xcode_clt,
            commands::system::install_homebrew,
            // CLI commands
            commands::cli::check_cli_installed,
            commands::cli::get_cli_version,
            commands::cli::get_latest_cli_version,
            commands::cli::install_cli,
            commands::cli::update_cli,
            commands::cli::check_auth_status,
            commands::cli::trigger_auth_login,
            commands::cli::auth_logout,
            // Plugin commands
            commands::plugin::check_plugin_installed,
            commands::plugin::get_plugin_version,
            commands::plugin::install_plugin,
            commands::plugin::update_plugin,
            commands::plugin::check_claude_code_installed,
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
