# Dyno Virtual Audio Mixer - Tauri Core Backend

Native Rust desktop integration and audio engine backend for Dyno Virtual Audio Mixer.

## 1. Overview & Purpose

`src-tauri` hosts the Tauri v2 backend and the native Windows audio routing engine. It acts as the bridge between low-level Windows CoreAudio / WASAPI multimedia endpoints and the high-level React Flow canvas UI.
It provides:
- Strongly-typed Tauri IPC commands (`commands.rs`) for hardware endpoint discovery, running application detection, virtual cable provisioning, and routing matrix manipulation.
- A high-throughput, low-latency background Tokio task broadcasting real-time 60 FPS peak meter levels (`audio-meter-tick`) to the web frontend via `tauri::Emitter`.
- Thread-safe, shared application state (`AppState`) managing the core routing matrix and virtual cable devices.

## 2. Prerequisites & Dependencies

- **Rust Compiler:** Stable `rustc` 1.75+ (MSVC ABI)
- **Windows SDK:** Windows 10/11 SDK with C++ Build Tools
- **Dependencies:**
  - `tauri` v2 with `tray-icon`
  - `tokio` v1 with `full`
  - `serde` & `serde_json`
  - `cpal` v0.15
  - `windows` v0.58

## 3. Tauri IPC Commands Reference

| Command | Arguments | Return Type | Description |
| :--- | :--- | :--- | :--- |
| `get_devices` | *none* | `Result<Vec<AudioDeviceDescriptor>, String>` | Enumerates physical and virtual audio hardware endpoints |
| `get_running_apps` | *none* | `Result<Vec<AudioProcessDescriptor>, String>` | Scans running processes with active WASAPI audio sessions |
| `get_virtual_devices` | *none* | `Result<Vec<VirtualDeviceDescriptor>, String>` | Lists all registered virtual audio cables and loopback endpoints |
| `create_virtual_device` | `name: String`, `cable_type: Option<String>` | `Result<VirtualDeviceDescriptor, String>` | Creates a new virtual audio cable or loopback endpoint |
| `remove_virtual_device` | `id: String` | `Result<bool, String>` | Removes a registered virtual audio device by its unique ID |
| `update_routing_graph` | `graph: AudioGraphDto` | `Result<bool, String>` | Replaces the active routing matrix with the updated graph |
| `set_node_volume` | `node_id: String`, `volume: f32`, `muted: bool` | `Result<bool, String>` | Updates gain and mute state for an individual mixer node |
| `start_meter_stream` | `app: AppHandle` | `()` | Spawns background ~16ms loop emitting `audio-meter-tick` events |

## 4. Real-Time Telemetry Event Schema (`audio-meter-tick`)

Emitted every ~16ms (60 FPS) to frontend listeners:

```json
{
  "timestamp_ms": 1700000000000,
  "meters": [
    {
      "node_id": "app_spotify",
      "peak_left": -12.5,
      "peak_right": -14.2
    },
    {
      "node_id": "mic_in",
      "peak_left": -60.0,
      "peak_right": -60.0
    }
  ]
}
```

## 5. CLI & Test Execution Examples

```powershell
# Type check and syntax validation
cargo check --manifest-path src-tauri/Cargo.toml

# Run IPC commands test suite
cargo test --manifest-path src-tauri/Cargo.toml --test test_commands

# Run complete regression test suite
cargo test --manifest-path src-tauri/Cargo.toml

# Run clippy static analysis
cargo clippy --manifest-path src-tauri/Cargo.toml
```

## 6. Output Structure & Schema

- `src/main.rs`: Windows executable entry point (hides console window in release).
- `src/lib.rs`: Library entry point configuring `tauri::Builder`, state management, and invoke handlers.
- `src/commands.rs`: IPC command handlers, DTOs (`AudioGraphDto`, `NodeDto`, `ConnectionDto`, `MeterUpdateDto`), and `AppState`.
- `src/audio/`: Core WASAPI audio processing engine, lock-free ring buffers, and device adapters.
- `tauri.conf.json`: Tauri v2 configuration (window size, title, devUrl, identifier).
- `capabilities/default.json`: Windows permissions and capability declarations.

## 7. Troubleshooting & Edge Cases

- **Missing Windows SDK:** Install "Desktop development with C++" in Visual Studio Installer.
- **Tauri State Lock Poisoning:** State locks use standard `std::sync::RwLock`; avoid panics inside critical sections.
- **Dropped Event Stream:** If the frontend window is closed or reloaded, `app.emit("audio-meter-tick")` logs a non-fatal warning without crashing the broadcaster task.
