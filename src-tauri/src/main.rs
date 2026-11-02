//! @file main.rs
//! @brief Executable binary entry point for Dyno Virtual Audio Mixer.
//! @details Disables console window in Windows release mode and transfers
//! execution to `dyno_virtual_audio_mixer_lib::run()`.

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// @brief Process entry point for the desktop executable.
/// @details Delegates application execution to the library run routine.
fn main() {
    dyno_virtual_audio_mixer_lib::run();
}
