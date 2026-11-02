### Task 6: Tauri IPC Commands & 60 FPS Peak Meter Streaming

**Files:**
- Create: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/src/commands.rs`
- Modify: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/src/lib.rs`
- Modify: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/src/main.rs`
- Test: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/tests/test_commands.rs`

**Interfaces:**
- Consumes: `src-tauri/src/audio/*`, `tauri::Manager`, `tauri::Emitter`
- Produces:
  - `AppState`: struct holding `Arc<RwLock<RoutingMatrix>>`, `Arc<RwLock<VirtualDeviceManager>>`, `Arc<AtomicBool>` running state
  - DTOs: `AudioGraphDto`, `NodeDto`, `ConnectionDto`, `MeterUpdateDto`
  - Tauri Commands:
    - `get_devices() -> Result<Vec<AudioDeviceDescriptor>, String>`
    - `get_running_apps() -> Result<Vec<AudioProcessDescriptor>, String>`
    - `get_virtual_devices() -> Result<Vec<VirtualDeviceDescriptor>, String>`
    - `create_virtual_device(name: String, cable_type: Option<String>) -> Result<VirtualDeviceDescriptor, String>`
    - `remove_virtual_device(id: String) -> Result<bool, String>`
    - `update_routing_graph(graph: AudioGraphDto) -> Result<bool, String>`
    - `set_node_volume(node_id: String, volume: f32, muted: bool) -> Result<bool, String>`
    - `start_meter_stream(app: AppHandle)`: Tokio background task emitting `audio-meter-tick` with `MeterUpdateDto` every ~16ms (60 FPS)

- [ ] **Step 1: Write unit tests for AppState and DTO serialization**
In `src-tauri/tests/test_commands.rs`:
  - Test serialization of `AudioGraphDto`, `NodeDto`, `ConnectionDto`, and `MeterUpdateDto`.
  - Test `AppState` creation and concurrency safety across multiple reader/writer threads.
  - Test conversion from DTO to `RoutingMatrix` and back.

- [ ] **Step 2: Run test to verify it fails**
Run `cargo test --manifest-path src-tauri/Cargo.toml --test test_commands` and observe failure.

- [ ] **Step 3: Implement Tauri command handlers and IPC state**
1. In `src-tauri/src/commands.rs`:
   - Implement `AppState`, all DTOs, and the Tauri commands.
   - Implement the background meter emitter task using `app.emit("audio-meter-tick", &meter_data)`.
2. In `src-tauri/src/lib.rs`:
   - Register commands with `.invoke_handler(tauri::generate_handler![...])`.
   - Setup `AppState` in `.manage(AppState::new())`.
   - Start meter task on app startup in `.setup(...)`.
3. In `src-tauri/src/main.rs`:
   - Ensure clean execution.
4. Document all functions with 100% Rustdoc Doxygen tags (`/// @brief`, `/// @param`, `/// @return`).

- [ ] **Step 4: Run test to verify it passes**
Run `cargo test --manifest-path src-tauri/Cargo.toml --test test_commands` and verify exit code 0.
Verify `cargo check --manifest-path src-tauri/Cargo.toml` finishes with exit code 0.

- [ ] **Step 5: Commit**
`git add src-tauri/ && git commit -m "feat(tauri): add ipc command handlers and 60fps vu-meter broadcaster"`
