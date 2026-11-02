### Task 4: Audio Routing Matrix & Real-Time Mixer Pipeline

**Files:**
- Create: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/src/audio/matrix.rs`
- Create: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/src/audio/mixer.rs`
- Modify: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/src/audio/mod.rs`
- Test: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/tests/test_matrix.rs`

**Interfaces:**
- Consumes: `AudioChunk`, `ChannelCount`, `SampleRate`
- Produces:
  - `AudioPinId`: struct with `node_id: String`, `channel_index: u16` (0 for L, 1 for R)
  - `ConnectionWire`: struct with `id: String`, `source: AudioPinId`, `destination: AudioPinId`, `gain: f32`, `muted: bool`
  - `MixerNode`: struct with `id: String`, `name: String`, `volume: f32`, `muted: bool`, `input_channels: u16`, `output_channels: u16`
  - `RoutingMatrix`: struct managing nodes, connections, volume scales, and executing multichannel sample routing:
    - `add_node(&mut self, node: MixerNode)`
    - `remove_node(&mut self, node_id: &str)`
    - `connect(&mut self, wire: ConnectionWire)`
    - `disconnect(&mut self, wire_id: &str)`
    - `process_frame(&self, inputs: &std::collections::HashMap<String, AudioChunk>, destination_node_id: &str) -> AudioChunk`
    - Channel routing operations: 1-to-1 direct routing, stereo downmix (0.5 * L + 0.5 * R), mono split to stereo, gain application, mute, and soft-limiter clipping.

- [ ] **Step 1: Write failing unit tests for routing matrix and mixer**
In `src-tauri/tests/test_matrix.rs`:
  - Test 1: Direct 1-to-1 routing (Source Node L/R -> Dest Node L/R) verifies exact sample propagation and volume gain.
  - Test 2: Stereo downmixing to mono (Source L/R -> Dest Mono) verifies (L + R) * 0.5.
  - Test 3: Mono channel split to stereo (Source Ch 0 -> Dest L and Dest R) verifies cloned channels.
  - Test 4: Mute verification: Muted node or wire outputs silence (0.0).
  - Test 5: Summing multiple sources with soft limiter prevents harsh digital clipping (> 1.0 clamped smoothly via `tanh`).

- [ ] **Step 2: Run test to verify it fails**
Run `cargo test --manifest-path src-tauri/Cargo.toml --test test_matrix` and observe failure.

- [ ] **Step 3: Implement RoutingMatrix and Mixer engine**
1. In `src-tauri/src/audio/matrix.rs`:
   - Implement `AudioPinId`, `ConnectionWire`, `MixerNode`, and `RoutingMatrix`.
   - Implement channel mapping and routing graph evaluation.
2. In `src-tauri/src/audio/mixer.rs`:
   - Implement `AudioMixerEngine`: manages audio inputs, active buffer queues, matrix processing, and produces output chunks for render endpoints.
3. Export in `src-tauri/src/audio/mod.rs`.
4. Ensure 100% Rustdoc Doxygen coverage (`/// @brief`, `/// @param`, `/// @return`).

- [ ] **Step 4: Run test to verify it passes**
Run `cargo test --manifest-path src-tauri/Cargo.toml --test test_matrix` and verify exit code 0.

- [ ] **Step 5: Commit**
`git add src-tauri/ && git commit -m "feat(audio): implement routing matrix and soft-clipping mixer engine"`
