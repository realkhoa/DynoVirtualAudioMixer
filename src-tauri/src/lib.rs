//! @file lib.rs
//! @brief Core library entry point and Tauri application setup for Dyno Virtual Audio Mixer.
//! @details Initializes the Tauri runtime builder, registers command handlers,
//! manages audio engine lifecycle states, initializes the system tray icon,
//! and handles background hide-on-close window lifecycle events.

/// @brief Core audio processing engine, lock-free ring buffers, and DSP primitives.
pub mod audio;

/// @brief Embedded HTTP REST API server for Stream Deck and OBS integration.
pub mod api_server;

/// @brief Tauri IPC command handlers, application state, DTOs, and 60 FPS meter broadcaster.
pub mod commands;

/// @brief Profile preset manager for disk persistence and retrieval of routing graphs.
pub mod presets;

/// @brief Windows notification system tray icon and background execution lifecycle.
pub mod tray;

use tauri::Manager;

pub use commands::{
    create_virtual_device, delete_profile, get_devices, get_running_apps, get_virtual_devices,
    list_profiles, load_profile, remove_virtual_device, save_profile, set_node_monitor_mode,
    set_node_volume, set_wire_config, start_meter_stream, update_routing_graph, AppState,
    AudioGraphDto, ConnectionDto, MeterUpdateDto, NodeDto, NodeMeterDto,
};
pub use presets::{ProfileManager, ProfilePreset};

/// @brief Initializes and runs the Dyno Virtual Audio Mixer Tauri desktop application.
/// @details Sets up the Tauri builder, attaches state containers, configures window events,
/// initializes the system tray icon, registers window close interception to hide in background,
/// starts the background 60 FPS meter broadcaster, registers IPC command invocations,
/// and starts the application event loop.
/// @return Returns `()` on clean application exit.
/// @note In case of a fatal startup error, panics with an informative diagnostic message.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .manage(commands::AppState::new())
        .setup(|app| {
            tray::setup_tray(app)?;
            commands::start_meter_stream(app.handle().clone());
            let state = app.state::<commands::AppState>();
            api_server::start_api_server(state.inner().clone(), api_server::DEFAULT_API_PORT);
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::handlers::get_devices,
            commands::handlers::get_running_apps,
            commands::handlers::get_virtual_devices,
            commands::handlers::create_virtual_device,
            commands::handlers::remove_virtual_device,
            commands::handlers::update_routing_graph,
            commands::handlers::set_node_volume,
            commands::handlers::set_wire_config,
            commands::handlers::set_node_monitor_mode,
            commands::handlers::start_meter_stream,
            commands::handlers::save_profile,
            commands::handlers::load_profile,
            commands::handlers::list_profiles,
            commands::handlers::delete_profile,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Dyno Virtual Audio Mixer application");
}
