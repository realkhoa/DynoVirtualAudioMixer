### Task 9: Preset Profiles & Windows System Tray

**Files:**
- Create: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/src/presets.rs`
- Create: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/src/tray.rs`
- Modify: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/src/commands/handlers.rs`
- Modify: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/src/commands/mod.rs`
- Modify: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/src/lib.rs`
- Test: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/tests/test_presets.rs`

**Interfaces:**
- Consumes: Tauri Tray Icon API (`tauri::tray`, `tauri::menu`), `std::path::PathBuf`, Serde JSON
- Produces:
  - `ProfilePreset`: struct with `id: String`, `name: String`, `created_at: String`, `graph: AudioGraphDto`
  - Profile Manager: saves/loads JSON presets in local app data directory (e.g. `presets/` directory)
  - Tauri Commands:
    - `save_profile(preset: ProfilePreset) -> Result<String, String>`
    - `load_profile(name: String) -> Result<ProfilePreset, String>`
    - `list_profiles() -> Result<Vec<String>, String>`
    - `delete_profile(name: String) -> Result<bool, String>`
  - Windows System Tray & Background Execution:
    - Custom tray icon with menu: "Show / Hide Dyno Mixer", "Mute All", "Exit"
    - Intercept `WindowEvent::CloseRequested` in Tauri to hide the window instead of killing the process, ensuring audio routing remains active seamlessly in background!
    - Double click or click tray icon toggles window visibility.

- [ ] **Step 1: Write unit tests for ProfilePreset serialization and manager**
In `src-tauri/tests/test_presets.rs`:
  - Test serialization and deserialization of `ProfilePreset`.
  - Test saving profile to disk and reloading it cleanly.
  - Test listing profiles in a directory.
  - Test deleting profile file.
  - Test error handling for invalid/corrupt profile files.

- [ ] **Step 2: Run test to verify it fails**
Run `cargo test --manifest-path src-tauri/Cargo.toml --test test_presets` and observe failure.

- [ ] **Step 3: Implement presets module and system tray**
1. In `src-tauri/src/presets.rs`:
   - Implement `ProfilePreset` and `ProfileManager`.
2. In `src-tauri/src/tray.rs`:
   - Implement `setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>>` with menu items ("Show Dyno Mixer", "Mute All Audio", "Exit") and tray click handler.
3. In `src-tauri/src/commands/handlers.rs`:
   - Add Tauri command handlers: `save_profile`, `load_profile`, `list_profiles`, `delete_profile`.
4. In `src-tauri/src/lib.rs`:
   - Register commands, call `setup_tray`, and add window event listener on `WindowEvent::CloseRequested` to `api.prevent_close()` and `window.hide()`.
5. Ensure 100% Rustdoc Doxygen coverage (`/// @brief`, `/// @param`, `/// @return`).

- [ ] **Step 4: Run test to verify it passes**
Run `cargo test --manifest-path src-tauri/Cargo.toml --test test_presets` and verify exit code 0.
Run `cargo check --manifest-path src-tauri/Cargo.toml` and verify compilation succeeds.

- [ ] **Step 5: Commit**
`git add src-tauri/ && git commit -m "feat(app): add profile presets manager and windows system tray integration"`
