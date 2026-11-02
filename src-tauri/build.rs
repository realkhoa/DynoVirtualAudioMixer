//! @file build.rs
//! @brief Cargo build script executing Tauri code generation and resource embedding.

/// @brief Build script entry point.
/// @details Invokes `tauri_build::build()` to compile Tauri assets, IPC bindings, and capabilities.
fn main() {
    tauri_build::build();
}
