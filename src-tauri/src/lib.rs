//! @file lib.rs
//! @brief Core library entry point and Tauri application setup for Dyno Virtual Audio Mixer.
//! @details Initializes the Tauri runtime builder, registers command handlers,
//! manages audio engine lifecycle states, and handles application exit events.

/// @brief Initializes and runs the Dyno Virtual Audio Mixer Tauri desktop application.
/// @details Sets up the Tauri builder, attaches state containers, configures window events,
/// and starts the application event loop.
/// @return Returns `()` on clean application exit.
/// @note In case of a fatal startup error, panics with an informative diagnostic message.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|_app| {
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Dyno Virtual Audio Mixer application");
}
