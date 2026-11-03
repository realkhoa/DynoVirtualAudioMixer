//! @file mod.rs
//! @brief Tauri IPC command handlers, application state container, and 60 FPS meter broadcaster.
//! @details Bridges the desktop web frontend with the native Rust WASAPI audio engine.
//! Exposes IPC commands for audio endpoint discovery, process scanning, virtual cable management,
//! routing matrix graph persistence, volume and mute configuration, and real-time 60 FPS peak meter telemetry.

pub mod dto;
pub mod handlers;
pub mod state;

pub use dto::{AudioGraphDto, ConnectionDto, MeterUpdateDto, NodeDto, NodeMeterDto};
pub use handlers::{
    create_virtual_device, delete_profile, get_devices, get_running_apps, get_virtual_devices,
    list_profiles, load_profile, remove_virtual_device, save_profile, set_node_monitor_mode,
    set_node_volume, set_wire_config, start_meter_stream, update_routing_graph,
};
pub use state::AppState;
