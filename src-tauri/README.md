# Dyno Virtual Audio Mixer - Tauri Core Backend

Native Rust desktop integration and audio engine backend for Dyno Virtual Audio Mixer.

## 1. Overview & Purpose

`src-tauri` hosts the Tauri v2 backend and the native Windows audio routing engine. It acts as the bridge between low-level Windows CoreAudio / WASAPI multimedia endpoints and the high-level React Flow canvas UI.

## 2. Prerequisites & Dependencies

- **Rust Compiler:** Stable `rustc` 1.75+ (MSVC ABI)
- **Windows SDK:** Windows 10/11 SDK with C++ Build Tools
- **Dependencies:**
  - `tauri` v2 with `tray-icon`
  - `tokio` v1
  - `serde` & `serde_json`

## 3. CLI Options & Arguments Reference

| Command | Purpose |
| :--- | :--- |
| `cargo check` | Fast type checking and syntax validation |
| `cargo build` | Builds the native library and executable |
| `cargo test` | Runs unit tests across engine modules |

## 4. Usage Examples

```powershell
# Check compilation from src-tauri
cargo check

# Run unit and integration tests
cargo test
```

## 5. Output Structure & Schema

- `src/main.rs`: Windows executable entry point (hides console window in release).
- `src/lib.rs`: Library entry point configuring `tauri::Builder`.
- `tauri.conf.json`: Tauri v2 configuration (window size, title, devUrl, identifier).
- `capabilities/default.json`: Windows permissions and capability declarations.

## 6. Troubleshooting & Edge Cases

- **Missing Windows SDK:** Install "Desktop development with C++" in Visual Studio Installer.
- **Tauri Schema Warnings:** Check that `$schema` references the valid Tauri v2 config URL.
