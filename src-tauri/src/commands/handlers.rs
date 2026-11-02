//! @file handlers.rs
//! @brief Tauri IPC command invocation handlers and background 60 FPS meter telemetry streamer.
//! @details Dispatches requests from the web frontend to device managers, process scanners,
//! routing matrix modifiers, and initiates the background peak meter event broadcast.

use crate::audio::devices::{list_audio_devices, AudioDeviceDescriptor};
use crate::audio::process_scanner::{list_audio_processes, AudioProcessDescriptor};
use crate::audio::virtual_device::{VirtualCableType, VirtualDeviceDescriptor};
use crate::commands::dto::AudioGraphDto;
use crate::commands::state::AppState;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State};

/// @brief Tauri command to enumerate all physical and virtual audio endpoints available on the system.
/// @return `Ok(Vec<AudioDeviceDescriptor>)` on discovery success, or `Err(String)` on hardware querying failure.
#[tauri::command]
pub fn get_devices() -> Result<Vec<AudioDeviceDescriptor>, String> {
    list_audio_devices()
}

/// @brief Tauri command to scan active audio sessions and applications running on the system.
/// @return `Ok(Vec<AudioProcessDescriptor>)` listing running applications, or `Err(String)` on failure.
#[tauri::command]
pub fn get_running_apps() -> Result<Vec<AudioProcessDescriptor>, String> {
    list_audio_processes()
}

/// @brief Tauri command to list all registered virtual audio cables and loopback endpoints.
/// @param state Tauri application state container.
/// @return `Ok(Vec<VirtualDeviceDescriptor>)` containing active virtual devices.
#[tauri::command]
pub fn get_virtual_devices(
    state: State<'_, AppState>,
) -> Result<Vec<VirtualDeviceDescriptor>, String> {
    let vdevs = state.read_virtual_devices()?;
    Ok(vdevs.list_devices())
}

/// @brief Tauri command to create and register a new virtual audio device cable.
/// @param state Tauri application state container.
/// @param name User-defined friendly name for the virtual audio device.
/// @param cable_type Optional driver technology string ("vbcable", "vac", or "software_loopback").
/// @return `Ok(VirtualDeviceDescriptor)` on successful creation, or `Err(String)` on validation error.
#[tauri::command]
pub fn create_virtual_device(
    state: State<'_, AppState>,
    name: String,
    cable_type: Option<String>,
) -> Result<VirtualDeviceDescriptor, String> {
    let driver = match cable_type.as_deref().map(|s| s.to_lowercase()) {
        Some(ref s) if s.contains("vb") => VirtualCableType::VBCable,
        Some(ref s) if s.contains("vac") || s.contains("virtualaudio") => {
            VirtualCableType::VirtualAudioCable
        }
        _ => VirtualCableType::SoftwareLoopbackEndpoint,
    };

    let mut vdevs = state.write_virtual_devices()?;
    vdevs.create_virtual_device(&name, driver)
}

/// @brief Tauri command to remove a registered virtual audio device by its unique ID.
/// @param state Tauri application state container.
/// @param id Unique identifier of the virtual device to delete.
/// @return `Ok(true)` if removed, `Ok(false)` if device was not found, or `Err(String)` on failure.
#[tauri::command]
pub fn remove_virtual_device(state: State<'_, AppState>, id: String) -> Result<bool, String> {
    let mut vdevs = state.write_virtual_devices()?;
    vdevs.remove_virtual_device(&id)
}

/// @brief Tauri command to overwrite the active routing matrix with an updated patchbay graph.
/// @param state Tauri application state container.
/// @param graph Updated AudioGraphDto sent from the frontend node graph canvas.
/// @return `Ok(true)` on successful update, or `Err(String)` on lock acquisition failure.
#[tauri::command]
pub fn update_routing_graph(
    state: State<'_, AppState>,
    graph: AudioGraphDto,
) -> Result<bool, String> {
    let mut matrix = state.write_matrix()?;
    *matrix = graph.to_routing_matrix();
    Ok(true)
}

/// @brief Tauri command to update volume gain and mute status for an individual mixer node.
/// @param state Tauri application state container.
/// @param node_id Unique identifier of the target mixer node.
/// @param volume Linear volume scale (0.0 to 1.5).
/// @param muted Boolean flag indicating whether the node is muted.
/// @return `Ok(true)` if node updated, `Ok(false)` if node not found, or `Err(String)` on lock error.
#[tauri::command]
pub fn set_node_volume(
    state: State<'_, AppState>,
    node_id: String,
    volume: f32,
    muted: bool,
) -> Result<bool, String> {
    let mut matrix = state.write_matrix()?;
    if let Some(node) = matrix.get_node_mut(&node_id) {
        node.volume = volume;
        node.muted = muted;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// @brief Starts a background Tokio task emitting real-time 60 FPS peak meter telemetry.
/// @details Spawns an asynchronous timer loop running at ~16ms intervals (60 FPS).
/// On each tick, extracts current peak decibel levels across all matrix nodes from `AppState`
/// and broadcasts them to the web frontend using `app.emit("audio-meter-tick", &meter_data)`.
/// @param app Tauri AppHandle used to access managed state and broadcast IPC events.
#[tauri::command]
pub fn start_meter_stream(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(16));
        loop {
            interval.tick().await;

            let state_opt = app.try_state::<AppState>();
            let state = match state_opt {
                Some(s) => s,
                None => break,
            };

            if !state.is_running.load(Ordering::Relaxed) {
                continue;
            }

            let meter_snapshot = state.generate_meter_snapshot();
            if let Err(err) = app.emit("audio-meter-tick", &meter_snapshot) {
                eprintln!("[MeterBroadcaster] Broadcast error: {err}");
            }
        }
    });
}
