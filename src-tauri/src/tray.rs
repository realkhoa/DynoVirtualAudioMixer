//! @file tray.rs
//! @brief Windows system tray icon setup and background lifecycle management.
//! @details Configures the desktop notification tray icon, context menu actions
//! (Show/Hide window, Mute All Audio, Exit), and click event handling for continuous background execution.

use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{App, Manager};

/// @brief Configures and attaches the Windows notification system tray icon.
/// @details Sets up the tray icon with menu actions ("Show Dyno Mixer", "Mute All Audio", "Exit"),
/// handles left click / double click to toggle window visibility, and ensures continuous
/// background execution when the main window is minimized or closed.
/// @param app Reference to the Tauri application instance during startup.
/// @return `Ok(())` on success, or `Err(Box<dyn std::error::Error>)` on configuration failure.
pub fn setup_tray(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let show_i = MenuItem::with_id(app, "show", "Show Dyno Mixer", true, None::<&str>)?;
    let mute_i = MenuItem::with_id(app, "mute", "Mute All Audio", true, None::<&str>)?;
    let exit_i = MenuItem::with_id(app, "exit", "Exit", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show_i, &mute_i, &exit_i])?;

    let mut builder = TrayIconBuilder::new()
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip("Dyno Virtual Audio Mixer");

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }

    builder
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.unminimize();
                    let _ = window.set_focus();
                }
            }
            "mute" => {
                if let Some(state) = app.try_state::<crate::commands::AppState>() {
                    if let Ok(mut matrix) = state.write_matrix() {
                        for node in matrix.nodes_mut().values_mut() {
                            node.muted = true;
                        }
                    }
                }
            }
            "exit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.unminimize();
                        let _ = window.set_focus();
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}
