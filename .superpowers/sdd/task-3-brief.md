### Task 3: Device Enumeration & Process Audio Detection

**Files:**
- Modify: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/Cargo.toml` (add `cpal = "0.15"`, `windows = { version = "0.58", features = ["Win32_Media_Audio", "Win32_System_Com", "Win32_Foundation", "Win32_System_ProcessStatus", "Win32_System_Threading"] }`)
- Create: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/src/audio/devices.rs`
- Create: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/src/audio/process_scanner.rs`
- Modify: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/src/audio/mod.rs`
- Test: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/tests/test_devices.rs`

**Interfaces:**
- Consumes: `cpal`, `windows` crate, `serde`, `serde_json`
- Produces:
  - `AudioDeviceType`: Enum (`Input`, `Output`, `Virtual`)
  - `AudioDeviceDescriptor`: struct with `id: String`, `name: String`, `device_type: AudioDeviceType`, `channels: u16`, `sample_rate: u32`, `is_default: bool`
  - `AudioProcessDescriptor`: struct with `pid: u32`, `name: String`, `title: String`, `has_audio_session: bool`
  - `list_audio_devices() -> Result<Vec<AudioDeviceDescriptor>, String>`
  - `list_audio_processes() -> Result<Vec<AudioProcessDescriptor>, String>`

- [ ] **Step 1: Write test for device and process descriptors**
In `src-tauri/tests/test_devices.rs`:
  - Test serialization and deserialization of `AudioDeviceDescriptor` and `AudioProcessDescriptor`
  - Test filtering input vs output devices
  - Test `list_audio_devices()` runs without panics on the current machine and returns at least 1 input and 1 output device (or handles headless environment gracefully)

- [ ] **Step 2: Run test to verify it fails**
Run `cargo test --manifest-path src-tauri/Cargo.toml --test test_devices` and observe failure.

- [ ] **Step 3: Implement device and process scanning**
1. Add dependencies in `src-tauri/Cargo.toml`.
2. In `src-tauri/src/audio/devices.rs`:
   - Implement `AudioDeviceType` and `AudioDeviceDescriptor`.
   - Implement `list_audio_devices()` using `cpal::default_host()`: enumerate input and output devices, find default devices, extract channels and supported sample rates.
3. In `src-tauri/src/audio/process_scanner.rs`:
   - Implement `AudioProcessDescriptor`.
   - Implement `list_audio_processes()` scanning active processes or mockable process list, detecting running applications like Spotify, Discord, Chrome, games.
4. Export modules in `src-tauri/src/audio/mod.rs`.
5. Ensure 100% Rustdoc Doxygen tags (`/// @brief`, `/// @param`, `/// @return`).

- [ ] **Step 4: Run test to verify it passes**
Run `cargo test --manifest-path src-tauri/Cargo.toml --test test_devices` and verify exit code 0.

- [ ] **Step 5: Commit**
`git add src-tauri/ && git commit -m "feat(audio): implement device enumeration and audio process detection"`
