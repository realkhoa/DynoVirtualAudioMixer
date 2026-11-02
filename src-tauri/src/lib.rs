//! @file lib.rs
//! @brief Core library entry point and Tauri application setup for Dyno Virtual Audio Mixer.
//! @details Initializes the Tauri runtime builder, registers command handlers,
//! manages audio engine lifecycle states, and handles application exit events.

/// @brief Core audio processing engine, lock-free ring buffers, and DSP primitives.
pub mod audio;

/// @brief Tauri IPC command handlers, application state, DTOs, and 60 FPS meter broadcaster.
pub mod commands;

pub use commands::{
    create_virtual_device, get_devices, get_running_apps, get_virtual_devices,
    remove_virtual_device, set_node_volume, start_meter_stream, update_routing_graph, AppState,
    AudioGraphDto, ConnectionDto, MeterUpdateDto, NodeDto, NodeMeterDto,
};

/// @brief Initializes and runs the Dyno Virtual Audio Mixer Tauri desktop application.
/// @details Sets up the Tauri builder, attaches state containers, configures window events,
/// starts the background 60 FPS meter broadcaster, registers IPC command invocations,
/// and starts the application event loop.
/// @return Returns `()` on clean application exit.
/// @note In case of a fatal startup error, panics with an informative diagnostic message.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(commands::AppState::new())
        .setup(|app| {
            commands::start_meter_stream(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::handlers::get_devices,
            commands::handlers::get_running_apps,
            commands::handlers::get_virtual_devices,
            commands::handlers::create_virtual_device,
            commands::handlers::remove_virtual_device,
            commands::handlers::update_routing_graph,
            commands::handlers::set_node_volume,
            commands::handlers::start_meter_stream,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Dyno Virtual Audio Mixer application");
}
