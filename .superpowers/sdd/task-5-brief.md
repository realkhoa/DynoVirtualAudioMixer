### Task 5: Virtual Audio Device Adapter & Bridge

**Files:**
- Create: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/src/audio/virtual_device.rs`
- Modify: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/src/audio/mod.rs`
- Test: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/tests/test_virtual_device.rs`

**Interfaces:**
- Consumes: `AudioDeviceDescriptor`, `AudioDeviceType`, `serde`, `serde_json`
- Produces:
  - `VirtualCableType`: Enum (`VBCable`, `VirtualAudioCable`, `SoftwareLoopbackEndpoint`)
  - `VirtualDeviceDescriptor`: struct with `id: String`, `name: String`, `driver_type: VirtualCableType`, `render_endpoint_id: Option<String>`, `capture_endpoint_id: Option<String>`, `channels: u16`, `sample_rate: u32`, `is_active: bool`
  - `VirtualDeviceManager`: struct managing virtual endpoints:
    - `new() -> Self`
    - `detect_available_drivers(&self) -> Vec<VirtualCableType>`
    - `create_virtual_device(&mut self, name: &str, cable_type: VirtualCableType) -> Result<VirtualDeviceDescriptor, String>`
    - `remove_virtual_device(&mut self, device_id: &str) -> Result<bool, String>`
    - `list_devices(&self) -> Vec<VirtualDeviceDescriptor>`
    - `get_device(&self, device_id: &str) -> Option<&VirtualDeviceDescriptor>`

- [ ] **Step 1: Write failing unit tests for VirtualDeviceManager**
In `src-tauri/tests/test_virtual_device.rs`:
  - Test 1: VirtualDeviceManager instantiation and driver detection returns known types or fallback software loopback.
  - Test 2: Creating a named virtual device registers descriptor with unique ID and active state.
  - Test 3: Duplicate device name handling or auto-incrementing naming.
  - Test 4: Removing virtual device by ID.
  - Test 5: Serialization of `VirtualDeviceDescriptor` to JSON for Tauri IPC.

- [ ] **Step 2: Run test to verify it fails**
Run `cargo test --manifest-path src-tauri/Cargo.toml --test test_virtual_device` and observe failure.

- [ ] **Step 3: Implement VirtualDeviceManager and adapter**
1. In `src-tauri/src/audio/virtual_device.rs`:
   - Implement `VirtualCableType`, `VirtualDeviceDescriptor`, and `VirtualDeviceManager`.
   - Implement scanning for Windows virtual audio devices (matching common driver names like "CABLE Input", "CABLE Output", "VB-Audio", "VoiceMeeter", "Virtual Audio Cable").
   - Implement software loopback sink bridge: if no physical driver is installed, create a virtual software routing sink endpoint that can be monitored or captured by loopback.
2. Export in `src-tauri/src/audio/mod.rs`.
3. Ensure 100% Rustdoc Doxygen coverage (`/// @brief`, `/// @param`, `/// @return`).

- [ ] **Step 4: Run test to verify it passes**
Run `cargo test --manifest-path src-tauri/Cargo.toml --test test_virtual_device` and verify exit code 0.

- [ ] **Step 5: Commit**
`git add src-tauri/ && git commit -m "feat(audio): implement virtual device adapter and cable manager"`
