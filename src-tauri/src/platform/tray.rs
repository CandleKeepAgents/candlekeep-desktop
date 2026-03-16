use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    App, Manager,
};

/// Set up the system tray icon and menu.
/// On macOS, uses template icon and hides from dock.
/// On Linux, attempts tray creation with fallback to visible window.
pub fn setup_tray(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let show_item = MenuItemBuilder::with_id("show", "Show CandleKeep").build(app)?;
    let quit_item = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
    let menu = MenuBuilder::new(app)
        .item(&show_item)
        .separator()
        .item(&quit_item)
        .build()?;

    let tray_icon_bytes = include_bytes!("../../icons/tray-icon@2x.png");
    let tray_image = image::load_from_memory(tray_icon_bytes)
        .expect("Failed to load tray icon")
        .into_rgba8();
    let (width, height) = tray_image.dimensions();
    let raw = tray_image.into_raw();
    let icon = tauri::image::Image::new(&raw, width, height);

    let mut tray_builder = TrayIconBuilder::new().icon(icon);

    // Template icon is macOS-only (renders correctly in light/dark menu bar)
    #[cfg(target_os = "macos")]
    {
        tray_builder = tray_builder.icon_as_template(true);
    }

    let result = tray_builder
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
        .build(app);

    match result {
        Ok(_) => {}
        Err(e) => {
            // On Linux, tray may fail if no system tray is available.
            // Fall back to showing the window directly.
            // On macOS, tray is essential — propagate the error.
            #[cfg(target_os = "macos")]
            {
                return Err(e.into());
            }
            // On Linux/Windows, tray may fail — fall back to showing the window directly.
            #[cfg(not(target_os = "macos"))]
            {
                tracing::warn!("Tray creation failed, showing window directly: {}", e);
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
                return Ok(());
            }
        }
    }

    // Hide from dock on macOS — but only after setup is complete so
    // first-time users can see the app in the dock.
    #[cfg(target_os = "macos")]
    {
        let state = crate::state::AppState::load();
        if state.setup_completed {
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
        }
    }

    Ok(())
}
