# Loopback Pro Audio Suite Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement 8 advanced professional audio routing and DSP capabilities inspired by Rogue Amoeba Loopback: Wire Gain & Phase Invert, Dedicated Monitors with Mute-When-Monitoring, Multi-Channel (4/8/16 ch) Virtual Devices, Audio Hotplug/Fallback, Real-Time Noise Gate, Sidechain Auto-Ducking, Remote Control REST API (Stream Deck/OBS), and Compact Mini Bar View.

**Architecture:** Extend Dyno's Rust audio engine (`matrix.rs`, `mixer.rs`, new `dsp/` submodule) with real-time frame processing algorithms (envelope followers, biquad/gain multipliers, tanh limiter), expand Tauri IPC DTOs and state management, and build rich interactive React Flow controls (Edge inspector popover, multi-channel VU meters, ducking modal, floating mini bar mode).

**Tech Stack:** Rust (WASAPI, CPAL, SPSC lock-free ring buffers), Tauri v2 IPC, TypeScript, React 19, React Flow (@xyflow/react), Tailwind CSS, Lucide icons, Bun runtime.

## Global Constraints

- 100% Doxygen / TSDoc coverage for all symbols (`@brief`, `@param`, `@return`, `@details`).
- Zero untested code: every DSP algorithm and IPC command must have automated unit tests.
- CI compatibility: all hardware and COM queries must gracefully bypass in headless CI environments.
- High performance: DSP calculations must run within sub-millisecond budgets per frame buffer without memory allocations in the audio thread.

---

### Task 1: Wire Gain Control, Wire Muting, and Phase Invert

**Files:**
- Modify: `src-tauri/src/audio/matrix.rs:16-95`
- Modify: `src-tauri/src/commands/dto.rs:50-95`
- Modify: `src-tauri/src/commands/handlers.rs:75-115`
- Test: `src-tauri/tests/test_matrix.rs`
- Test: `src-tauri/tests/test_commands.rs`

**Interfaces:**
- Consumes: `ConnectionWire`, `AudioChunk`, `RoutingMatrix`
- Produces: `ConnectionWire::with_phase_invert(bool)`, `set_wire_gain_and_phase(wire_id, gain, muted, phase_invert)`

- [ ] **Step 1: Write failing test in `test_matrix.rs` for phase inversion and wire gain multiplier**

```rust
#[test]
fn test_wire_phase_inversion_and_attenuation() {
    let mut matrix = RoutingMatrix::new();
    let src = MixerNode::new("src_tone", "Tone Generator", 0, 1).with_volume(1.0);
    let dst = MixerNode::new("dst_out", "Output", 1, 1).with_volume(1.0);
    matrix.add_node(src);
    matrix.add_node(dst);

    let mut wire = ConnectionWire::new("wire_inverted", AudioPinId::new("src_tone", 0), AudioPinId::new("dst_out", 0), 0.5);
    wire.phase_invert = true;
    matrix.connect(wire);

    let mut engine = AudioMixerEngine::new(48000);
    engine.matrix_mut().clone_from(&matrix);
    let chunk = AudioChunk::new(48000, 1, vec![0.8, -0.4, 0.2]);
    engine.submit_input("src_tone", chunk);

    let output_chunk = engine.render_output("dst_out").expect("Output chunk must exist");
    // Expected: 0.8 * 0.5 * -1.0 = -0.4; -0.4 * 0.5 * -1.0 = 0.2; 0.2 * 0.5 * -1.0 = -0.1
    assert!((output_chunk.samples()[0] - (-0.4)).abs() < 1e-4);
    assert!((output_chunk.samples()[1] - 0.2).abs() < 1e-4);
    assert!((output_chunk.samples()[2] - (-0.1)).abs() < 1e-4);
}
```

- [ ] **Step 2: Update `ConnectionWire` in `src-tauri/src/audio/matrix.rs` to add `pub phase_invert: bool`**
- [ ] **Step 3: Update `AudioMixerEngine::render_output()` in `src-tauri/src/audio/mixer.rs` to multiply samples by `-1.0` when `wire.phase_invert` is true**
- [ ] **Step 4: Update `ConnectionDto` in `src-tauri/src/commands/dto.rs` with `pub phase_invert: Option<bool>` and DTO conversions**
- [ ] **Step 5: Run `cargo test --test test_matrix --test test_commands` and verify exit code 0**
- [ ] **Step 6: Commit changes:** `git commit -am "feat(audio): add wire gain, muting, and phase inversion to routing matrix"`

---

### Task 2: Dedicated Monitors and "Mute When Monitoring" Routing

**Files:**
- Modify: `src-tauri/src/audio/matrix.rs`
- Modify: `src-tauri/src/audio/mixer.rs`
- Modify: `src-tauri/src/commands/dto.rs`
- Modify: `src-tauri/src/commands/handlers.rs`
- Test: `src-tauri/tests/test_matrix.rs`

**Interfaces:**
- Consumes: `MixerNode`, `AudioMixerEngine`
- Produces: `MixerNode::is_monitor: bool`, `MixerNode::mute_when_monitoring: bool`

- [ ] **Step 1: Write failing test in `test_matrix.rs` for Mute When Monitoring behavior**

```rust
#[test]
fn test_mute_when_monitoring_behavior() {
    let mut matrix = RoutingMatrix::new();
    let app_spotify = MixerNode::new("app_spotify", "Spotify", 0, 2);
    let mut monitor_headphones = MixerNode::new("out_headphones", "Headphones Monitor", 2, 0);
    monitor_headphones.is_monitor = true;
    monitor_headphones.mute_when_monitoring = true;
    
    let stream_virtual = MixerNode::new("out_stream", "OBS Stream Out", 2, 0);

    matrix.add_node(app_spotify);
    matrix.add_node(monitor_headphones);
    matrix.add_node(stream_virtual);

    // Connect Spotify -> OBS Stream and Spotify -> Headphones Monitor
    matrix.connect(ConnectionWire::new("wire_stream", AudioPinId::new("app_spotify", 0), AudioPinId::new("out_stream", 0), 1.0));
    matrix.connect(ConnectionWire::new("wire_monitor", AudioPinId::new("app_spotify", 0), AudioPinId::new("out_headphones", 0), 1.0));

    let mut engine = AudioMixerEngine::new(48000);
    engine.matrix_mut().clone_from(&matrix);
    engine.submit_input("app_spotify", AudioChunk::new(48000, 2, vec![0.5, 0.5]));

    let monitor_out = engine.render_output("out_headphones").expect("Monitor output must exist");
    assert!(!monitor_out.is_silent());

    let stream_out = engine.render_output("out_stream").expect("Stream output must exist");
    assert!(!stream_out.is_silent());
}
```

- [ ] **Step 2: Add `is_monitor` and `mute_when_monitoring` fields to `MixerNode` with complete Doxygen comments**
- [ ] **Step 3: Update `NodeDto` in `src-tauri/src/commands/dto.rs` to expose `is_monitor` and `mute_when_monitoring`**
- [ ] **Step 4: Implement `set_node_monitor_mode(node_id, is_monitor, mute_when_monitoring)` IPC handler**
- [ ] **Step 5: Run tests and verify exit code 0**
- [ ] **Step 6: Commit changes:** `git commit -am "feat(audio): support dedicated monitors and mute-when-monitoring"`

---

### Task 3: Multi-Channel (4, 8, 16 Kênh) Virtual Devices & Dynamic Pins

**Files:**
- Modify: `src-tauri/src/audio/virtual_device.rs`
- Modify: `src-tauri/src/commands/handlers.rs`
- Modify: `src-tauri/src/commands/dto.rs`
- Test: `src-tauri/tests/test_virtual_device.rs`

**Interfaces:**
- Consumes: `VirtualDeviceManager`, `VirtualCableType`
- Produces: `create_virtual_device(name, cable_type, channels)`

- [ ] **Step 1: Write failing test in `test_virtual_device.rs` verifying 4-ch and 8-ch virtual cable creation**

```rust
#[test]
fn test_multichannel_virtual_device_creation() {
    let mut manager = VirtualDeviceManager::new();
    let desc_4ch = manager.create_virtual_device_with_channels(
        "Podcast Quad Out",
        VirtualCableType::SoftwareLoopbackEndpoint,
        4,
    ).expect("4-channel virtual device should be created");
    assert_eq!(desc_4ch.channels, 4);

    let desc_8ch = manager.create_virtual_device_with_channels(
        "Studio MultiTrack",
        VirtualCableType::SoftwareLoopbackEndpoint,
        8,
    ).expect("8-channel virtual device should be created");
    assert_eq!(desc_8ch.channels, 8);
}
```

- [ ] **Step 2: Implement `create_virtual_device_with_channels()` in `VirtualDeviceManager`**
- [ ] **Step 3: Update `create_virtual_device` IPC command to accept optional `channels: Option<u16>` (defaults to 2)**
- [ ] **Step 4: Run `cargo test --test test_virtual_device` and verify pass**
- [ ] **Step 5: Commit changes:** `git commit -am "feat(vdev): support 4, 8, and 16 multi-channel virtual cable endpoints"`

---

### Task 4: Real-Time Noise Gate DSP Engine

**Files:**
- Create: `src-tauri/src/audio/dsp/mod.rs`
- Create: `src-tauri/src/audio/dsp/gate.rs`
- Modify: `src-tauri/src/audio/mod.rs`
- Modify: `src-tauri/src/audio/mixer.rs`
- Test: `src-tauri/tests/test_dsp_gate.rs`

**Interfaces:**
- Consumes: `AudioChunk`, sample buffers
- Produces: `NoiseGate::new(threshold_db, attack_ms, release_ms, sample_rate)`, `NoiseGate::process(&mut samples)`

- [ ] **Step 1: Write failing unit tests in `test_dsp_gate.rs` validating signal attenuation below threshold and pass-through above threshold**

```rust
#[test]
fn test_noise_gate_attenuation_below_threshold() {
    let mut gate = NoiseGate::new(-40.0, 1.0, 50.0, 48000);
    let mut quiet_samples = vec![0.001f32; 480]; // ~ -60 dBFS
    gate.process(&mut quiet_samples);
    for sample in quiet_samples {
        assert!(sample.abs() < 1e-4, "Quiet signal should be gated to near zero");
    }

    let mut loud_samples = vec![0.5f32; 480]; // ~ -6 dBFS
    gate.process(&mut loud_samples);
    assert!(loud_samples[479] > 0.45, "Loud signal should pass through gate cleanly");
}
```

- [ ] **Step 2: Implement `NoiseGate` state machine in `src-tauri/src/audio/dsp/gate.rs` with smooth envelope follower**
- [ ] **Step 3: Expose `NoiseGate` in `src-tauri/src/audio/mod.rs` and attach optional gate instance to `MixerNode`**
- [ ] **Step 4: Run tests and verify exit code 0**
- [ ] **Step 5: Commit changes:** `git commit -am "feat(dsp): implement real-time noise gate filter with envelope detection"`

---

### Task 5: Real-Time Sidechain Auto-Ducking DSP Engine

**Files:**
- Create: `src-tauri/src/audio/dsp/ducker.rs`
- Modify: `src-tauri/src/audio/dsp/mod.rs`
- Modify: `src-tauri/src/audio/mixer.rs`
- Modify: `src-tauri/src/commands/dto.rs`
- Modify: `src-tauri/src/commands/handlers.rs`
- Test: `src-tauri/tests/test_dsp_ducker.rs`

**Interfaces:**
- Consumes: `AudioChunk`
- Produces: `AutoDuckerConfig`, `AutoDucker::process_sidechain(trigger_chunk, &mut target_chunks)`

- [ ] **Step 1: Write failing test in `test_dsp_ducker.rs` verifying target attenuation when trigger exceeds threshold**

```rust
#[test]
fn test_auto_ducking_attenuates_music_when_voice_active() {
    let mut ducker = AutoDucker::new(
        "mic_host",
        vec!["app_spotify".to_string()],
        -30.0, // threshold dB
        -18.0, // duck reduction dB
        10.0,  // attack ms
        200.0, // release ms
        48000,
    );

    let mic_loud = AudioChunk::new(48000, 1, vec![0.4f32; 480]); // Voice active ~ -8 dBFS
    let mut spotify_music = AudioChunk::new(48000, 2, vec![0.8f32; 960]);

    ducker.process(&mic_loud, &mut [(&"app_spotify".to_string(), &mut spotify_music)]);
    let last_sample = spotify_music.samples()[959];
    // -18 dB reduction corresponds to ~0.125 multiplier: 0.8 * 0.125 = ~0.1
    assert!(last_sample < 0.2, "Music should be ducked below 0.2 during voice activity");
}
```

- [ ] **Step 2: Implement `AutoDucker` in `src-tauri/src/audio/dsp/ducker.rs` with envelope detector and smooth gain ramp**
- [ ] **Step 3: Integrate `AutoDucker` into `AudioMixerEngine::process_frame()`**
- [ ] **Step 4: Expose `set_ducker_config` IPC command in `src-tauri/src/commands/handlers.rs`**
- [ ] **Step 5: Run tests and verify exit code 0**
- [ ] **Step 6: Commit changes:** `git commit -am "feat(dsp): implement sidechain auto-ducking for streamer voice priority"`

---

### Task 6: Audio Hotplug Watchdog & Fallback Routing

**Files:**
- Modify: `src-tauri/src/audio/devices.rs`
- Modify: `src-tauri/src/audio/matrix.rs`
- Modify: `src-tauri/src/commands/state.rs`
- Test: `src-tauri/tests/test_devices.rs`

**Interfaces:**
- Consumes: `list_audio_devices`
- Produces: `RoutingMatrix::mark_device_offline(id, bool)`, `RoutingMatrix::set_fallback_device(Option<String>)`

- [ ] **Step 1: Write failing test in `test_matrix.rs` verifying automatic redirection to fallback device when primary device is offline**

```rust
#[test]
fn test_fallback_routing_when_primary_offline() {
    let mut matrix = RoutingMatrix::new();
    let src = MixerNode::new("src_app", "Game Audio", 0, 2);
    let mut dst_primary = MixerNode::new("dst_usb_headset", "USB Headset", 2, 0);
    let dst_fallback = MixerNode::new("dst_speakers", "Default Speakers", 2, 0);

    matrix.add_node(src);
    matrix.add_node(dst_primary.clone());
    matrix.add_node(dst_fallback);

    matrix.set_fallback_node_id(Some("dst_speakers".to_string()));
    matrix.connect(ConnectionWire::new("wire_1", AudioPinId::new("src_app", 0), AudioPinId::new("dst_usb_headset", 0), 1.0));

    // Mark primary device offline
    matrix.set_node_offline("dst_usb_headset", true);

    let mut engine = AudioMixerEngine::new(48000);
    engine.matrix_mut().clone_from(&matrix);
    engine.submit_input("src_app", AudioChunk::new(48000, 2, vec![0.7, 0.7]));

    let fallback_out = engine.render_output("dst_speakers").expect("Fallback output should receive rerouted audio");
    assert!(!fallback_out.is_silent());
}
```

- [ ] **Step 2: Add `is_offline` and `fallback_node_id` support to `RoutingMatrix` and `AudioMixerEngine`**
- [ ] **Step 3: Run tests and verify exit code 0**
- [ ] **Step 4: Commit changes:** `git commit -am "feat(audio): implement audio device hotplug fallback rerouting"`

---

### Task 7: Lightweight Control REST API for Stream Deck & OBS (Port 49152)

**Files:**
- Create: `src-tauri/src/api_server.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/Cargo.toml`
- Test: `src-tauri/tests/test_api_server.rs`

**Interfaces:**
- Consumes: `AppState`
- Produces: HTTP endpoints `GET /api/status`, `POST /api/mute`, `POST /api/profile`, `POST /api/volume`

- [ ] **Step 1: Write integration test in `test_api_server.rs` querying `/api/status` and posting `/api/mute`**
- [ ] **Step 2: Implement lightweight background TCP/HTTP server in `src-tauri/src/api_server.rs` bound to `127.0.0.1:49152`**
- [ ] **Step 3: Spawn API server in `src-tauri/src/lib.rs` alongside Tauri app setup**
- [ ] **Step 4: Run tests and verify exit code 0**
- [ ] **Step 5: Commit changes:** `git commit -am "feat(api): embed lightweight localhost HTTP server for Stream Deck and OBS"`

---

### Task 8: Frontend UI — Edge Inspector, Multi-Channel Pins & Ducking Modal

**Files:**
- Create: `src/components/edges/EdgeInspectorModal.tsx`
- Create: `src/components/modals/DuckingModal.tsx`
- Modify: `src/types/audio.ts`
- Modify: `src/components/edges/GlowingEdge.tsx`
- Modify: `src/components/nodes/AudioNode.tsx`
- Modify: `src/components/nodes/VuMeter.tsx`
- Modify: `src/components/header/Header.tsx`
- Test: `src/components/edges/GlowingEdge.test.ts`
- Test: `src/types/audio.test.ts`

**Interfaces:**
- Consumes: Backend DTOs (`phase_invert`, `gain`, `is_monitor`, `mute_when_monitoring`, multi-channel counts)
- Produces: Interactive Edge clicking, popover controls, multi-channel VU meters, ducking setup modal

- [ ] **Step 1: Write failing frontend tests in `src/components/edges/GlowingEdge.test.ts` for edge inspection and phase inversion styling**
- [ ] **Step 2: Update `src/types/audio.ts` with TypeScript contracts for wire gain, phase invert, monitor flags, and ducking configuration**
- [ ] **Step 3: Implement `EdgeInspectorModal.tsx` allowing users to click any wire and adjust Wire Gain (-60dB to +6dB), Mute, and Phase Invert (`Ø`)**
- [ ] **Step 4: Update `AudioNode.tsx` and `VuMeter.tsx` to dynamically render multi-channel pin stacks (2, 4, 8, 16 pins) with channel labels (`Ch 1`, `Ch 2`, etc.)**
- [ ] **Step 5: Implement `DuckingModal.tsx` accessible from the Header with Trigger/Target selection, threshold slider, and ducking depth**
- [ ] **Step 6: Run `bun test` and verify all tests pass with 0 failures**
- [ ] **Step 7: Commit changes:** `git commit -am "feat(ui): add edge inspector popover, multi-channel pin stacks, and ducking modal"`

---

### Task 9: Frontend UI — Compact Floating Mini Bar Mode

**Files:**
- Create: `src/components/compact/MiniBar.tsx`
- Modify: `src/App.tsx`
- Modify: `src/components/header/Header.tsx`
- Test: `src/components/compact/MiniBar.test.ts`

**Interfaces:**
- Consumes: `useAudioGraph`, `useMeterStream`
- Produces: Compact floating widget UI, Tauri window resize to 380x200 with always-on-top toggle

- [ ] **Step 1: Write unit test in `src/components/compact/MiniBar.test.ts` testing active source display and quick mute toggles**
- [ ] **Step 2: Implement `MiniBar.tsx` rendering a high-density, horizontal audio meter bar with app icons, master volume, and expand button**
- [ ] **Step 3: Add "Compact Mode" toggle in `Header.tsx` that resizes the Tauri window and toggles between Canvas and MiniBar**
- [ ] **Step 4: Run `bun test` and `bun run build` to verify clean compilation**
- [ ] **Step 5: Commit changes:** `git commit -am "feat(ui): implement compact floating mini bar mode for streaming and gaming"`

---

### Task 10: E2E Full Integration & Verification Pass Gate

**Files:**
- Test: `src-tauri/tests/e2e_audio_smoke_test.rs`
- Run: Full CI regression test suite (`bun test`, `cargo test`, `bun run build`)

- [ ] **Step 1: Run complete frontend unit tests:** `bun test`
- [ ] **Step 2: Run complete backend audio engine tests:** `cargo test --manifest-path src-tauri/Cargo.toml -- --test-threads=1`
- [ ] **Step 3: Run production frontend build:** `bun run build`
- [ ] **Step 4: Execute deterministic static review / verification check**
- [ ] **Step 5: Commit and push all changes to GitHub**
