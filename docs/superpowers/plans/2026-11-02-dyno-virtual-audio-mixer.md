# Dyno Virtual Audio Mixer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a lightweight (<50MB RAM), high-performance (sub-10ms latency), and visually sleek (Loopback-style) audio in/out routing patchbay desktop application on Windows using Tauri v2, Rust audio engine (WASAPI & lock-free ring buffers), and React Flow with live animated signal cables.

**Architecture:**
- **Frontend Layer (UI):** Tauri v2 WebView2 + React 19 + Vite + Tailwind CSS + React Flow. Interactive node canvas featuring custom Source, Pass-through, and Output nodes, live Peak VU meters (60 FPS), volume sliders, and glowing Bezier patch cords.
- **IPC & Bridge Layer:** Tauri IPC commands (`invoke`) and high-throughput events/channels (`emit`) for pushing real-time audio levels and device topology between Rust and React.
- **Audio Core Engine (Rust):** Lock-free SPSC ring buffers (`rtrb`), WASAPI hardware device capture & process-level loopback (`windows-rs` / `cpal`), dynamic channel routing matrix, stereo downmix/split, and low-latency audio mixing pipeline.
- **Virtual Device Module:** Integration with Windows virtual audio cable endpoints (WDM/Sysvad or VB-Cable driver interfaces) enabling external applications (Discord, Zoom, OBS) to receive custom mixed audio.

**Tech Stack:**
- Frontend: Bun, React, TypeScript, Tailwind CSS, `@xyflow/react` (React Flow), Lucide Icons
- Backend / Desktop: Tauri v2 (`@tauri-apps/cli`, `tauri` crate)
- Audio Engine: Rust (`cpal`, `windows-rs`, `rtrb`)
- Target Platform: Windows 10/11 x64

## Global Constraints
- Strictly enforce Vietnamese for all user communication and status reporting.
- Strictly enforce English for all code, identifiers, comments, tests, and documentation.
- 100% Rustdoc / Doxygen coverage on all functions, structs, traits, and exported methods (`@brief`, `@param`, `@return`).
- Zero untested core logic: All audio routing algorithms, channel matrix operations, and ring buffers must have unit tests.
- Verify clean compilation with exit code 0 (`cargo test`, `bun test`, `bun run build`) before completing tasks.

---

### Task 1: Repository Initialization & Tauri v2 Scaffolding

**Files:**
- Create: `e:/sourcecode/DynoVirtualAudioMixer/.gitignore`
- Create: `e:/sourcecode/DynoVirtualAudioMixer/package.json`
- Create: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/Cargo.toml`
- Create: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/tauri.conf.json`
- Create: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/src/main.rs`
- Create: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/src/lib.rs`
- Create: `e:/sourcecode/DynoVirtualAudioMixer/vite.config.ts`
- Create: `e:/sourcecode/DynoVirtualAudioMixer/tsconfig.json`
- Create: `e:/sourcecode/DynoVirtualAudioMixer/index.html`
- Create: `e:/sourcecode/DynoVirtualAudioMixer/src/main.tsx`
- Create: `e:/sourcecode/DynoVirtualAudioMixer/src/App.tsx`
- Create: `e:/sourcecode/DynoVirtualAudioMixer/README.md`

**Interfaces:**
- Consumes: System runtimes (`bun`, `cargo`, Windows SDK)
- Produces: Runnable Tauri v2 skeleton with modern dark layout and Bun dev server

- [ ] **Step 1: Initialize Git and .gitignore**
Initialize repository and create `.gitignore` ignoring `target/`, `node_modules/`, `dist/`, `.turbo/`, and `.DS_Store`.

- [ ] **Step 2: Scaffold Bun + Vite + React + Tailwind + Tauri v2 project**
Setup `package.json`, `tsconfig.json`, `vite.config.ts`, and install dependencies (`@tauri-apps/api`, `@tauri-apps/cli`, `@xyflow/react`, `lucide-react`, `clsx`, `tailwind-merge`).

- [ ] **Step 3: Setup Tauri v2 backend configuration**
Configure `src-tauri/tauri.conf.json` with app identity `com.dyno.audiomixer`, window dimensions `1200x800`, transparent dark title bar, and permissions. Setup `src-tauri/Cargo.toml` with `tauri = { version = "2", features = ["tray-icon"] }`, `serde`, `serde_json`, `tokio`.

- [ ] **Step 4: Verify initial build and tests**
Run `bun run build` and `cargo check --manifest-path src-tauri/Cargo.toml` to verify zero errors.

- [ ] **Step 5: Commit**
`git add . && git commit -m "chore: scaffold tauri v2 project with react and rust backend"`

---

### Task 2: Core Audio Engine - Lock-Free Ring Buffer & Buffer Types

**Files:**
- Create: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/src/audio/mod.rs`
- Create: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/src/audio/types.rs`
- Create: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/src/audio/ring_buffer.rs`
- Test: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/tests/test_ring_buffer.rs`

**Interfaces:**
- Consumes: `rtrb` crate
- Produces: `AudioFrame`, `AudioChunk`, `LockFreeAudioQueue`, `SampleRate`, `ChannelCount`

- [ ] **Step 1: Write failing unit test for ring buffer queue**
Write test verifying multi-threaded single-producer single-consumer audio transfer without sample loss or mutex deadlock.

- [ ] **Step 2: Run test to verify it fails**
Run `cargo test --test test_ring_buffer` and observe failure.

- [ ] **Step 3: Implement lock-free audio buffer types**
Implement `AudioChunk` struct with interleaved float32 samples, peak level calculation (`calculate_peak_db`), and generic `AudioRingBuffer` wrapper around `rtrb::RingBuffer`. Document with full Rustdoc tags.

- [ ] **Step 4: Run test to verify it passes**
Run `cargo test --test test_ring_buffer` and verify exit code 0.

- [ ] **Step 5: Commit**
`git add src-tauri/src/audio && git commit -m "feat(audio): implement lock-free ring buffer and audio chunk primitives"`

---

### Task 3: Device Enumeration & Process Audio Detection

**Files:**
- Create: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/src/audio/devices.rs`
- Create: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/src/audio/process_scanner.rs`
- Test: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/tests/test_devices.rs`

**Interfaces:**
- Consumes: Windows CoreAudio / WASAPI APIs (`cpal`, `windows::Win32::Media::Audio`)
- Produces: `list_audio_devices() -> Vec<AudioDeviceDescriptor>`, `list_audio_processes() -> Vec<AudioProcessDescriptor>`

- [ ] **Step 1: Write test for device descriptor serialization**
Write unit test checking JSON serialization of input, output, and process descriptors.

- [ ] **Step 2: Implement hardware device enumerator**
Use `cpal::traits::HostTrait` to enumerate all active physical input and output audio endpoints on Windows (WASAPI host). Retrieve friendly names, default status, channel count, and sample rates.

- [ ] **Step 3: Implement active audio processes scanner**
Enumerate running processes holding active audio sessions via `IAudioSessionManager2` / `IAudioSessionEnumerator` or Windows Toolhelp32 process snapshotting, matching process name and icon.

- [ ] **Step 4: Verify test suite execution**
Run `cargo test --test test_devices` and verify PASS with exit code 0.

- [ ] **Step 5: Commit**
`git add src-tauri/src/audio/devices.rs src-tauri/src/audio/process_scanner.rs && git commit -m "feat(audio): add device enumeration and audio process detection"`

---

### Task 4: Audio Routing Matrix & Real-Time Mixer Pipeline

**Files:**
- Create: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/src/audio/matrix.rs`
- Create: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/src/audio/mixer.rs`
- Test: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/tests/test_matrix.rs`

**Interfaces:**
- Consumes: `AudioChunk`, `ChannelCount`
- Produces: `RoutingMatrix::route_and_mix()`, `MixerNode`, `ConnectionWire`

- [ ] **Step 1: Write failing unit tests for routing matrix**
Write unit tests covering:
  1. Direct 1-to-1 routing (Source L -> Dest L, Source R -> Dest R)
  2. Stereo downmixing to mono (0.5 * L + 0.5 * R)
  3. Channel splitting (Mono input cloned to L and R)
  4. Gain scaling and muting

- [ ] **Step 2: Implement RoutingMatrix and Channel Remapper**
Implement `RoutingMatrix` with dynamic pin connections, gain multipliers, mute states, and SIMD/vectorized float32 sample addition.

- [ ] **Step 3: Implement multi-source real-time audio mixer**
Implement the mixer processing loop with soft-clipping / limiter protection to prevent harsh digital clipping when multiple sources are summed together.

- [ ] **Step 4: Run test to verify it passes**
Run `cargo test --test test_matrix` and verify all matrix test cases pass.

- [ ] **Step 5: Commit**
`git add src-tauri/src/audio/matrix.rs src-tauri/src/audio/mixer.rs && git commit -m "feat(audio): implement routing matrix and soft-clipping mixer engine"`

---

### Task 5: Virtual Audio Device Adapter & Bridge

**Files:**
- Create: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/src/audio/virtual_device.rs`
- Test: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/tests/test_virtual_device.rs`

**Interfaces:**
- Consumes: Windows Audio API
- Produces: `VirtualDeviceManager::create_device()`, `VirtualDeviceManager::list_virtual_cables()`

- [ ] **Step 1: Write tests for virtual device configuration model**
Write test validating virtual cable mapping, friendly names, and fallback behaviors when physical or driver endpoints change.

- [ ] **Step 2: Implement VirtualDeviceManager**
Implement adapter supporting:
  1. Detection and linking to installed virtual cables (VB-Cable, VAC, Hi-Fi Cable).
  2. Software loopback sink creation: Expose a dedicated render stream that acts as a virtual pass-through.
  3. Configuration interface for spawning named routing endpoints.

- [ ] **Step 3: Run test suite**
Run `cargo test --test test_virtual_device` and verify all tests pass.

- [ ] **Step 4: Commit**
`git add src-tauri/src/audio/virtual_device.rs && git commit -m "feat(audio): implement virtual device adapter and cable manager"`

---

### Task 6: Tauri IPC Commands & 60 FPS Peak Meter Streaming

**Files:**
- Create: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/src/commands.rs`
- Modify: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/src/lib.rs`
- Modify: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/src/main.rs`

**Interfaces:**
- Consumes: Audio Engine, `tauri::AppHandle`, `tauri::ipc::Channel`
- Produces:
  - `get_devices()`
  - `get_running_apps()`
  - `update_routing_graph(graph: AudioGraphDto)`
  - `set_node_volume(node_id: String, volume: f32, muted: bool)`
  - Real-time `meter_update` event stream at 60 FPS containing peak dB per node pin

- [ ] **Step 1: Implement Tauri command handlers**
Expose strongly-typed Tauri commands with Serde JSON serialization.

- [ ] **Step 2: Implement throttled 60 FPS meter broadcaster**
Implement background Tokio task polling peak levels from active ring buffers and broadcasting to frontend via `app.emit("audio-meter-tick", meter_data)`.

- [ ] **Step 3: Verify Tauri build and command bindings**
Run `cargo check --manifest-path src-tauri/Cargo.toml` to ensure error-free compilation.

- [ ] **Step 4: Commit**
`git add src-tauri/src/commands.rs src-tauri/src/lib.rs && git commit -m "feat(tauri): add ipc command handlers and 60fps vu-meter broadcaster"`

---

### Task 7: Frontend Node Graph Canvas (React Flow + Custom Nodes)

**Files:**
- Create: `e:/sourcecode/DynoVirtualAudioMixer/src/types/audio.ts`
- Create: `e:/sourcecode/DynoVirtualAudioMixer/src/components/nodes/SourceNode.tsx`
- Create: `e:/sourcecode/DynoVirtualAudioMixer/src/components/nodes/OutputNode.tsx`
- Create: `e:/sourcecode/DynoVirtualAudioMixer/src/components/nodes/VirtualDeviceNode.tsx`
- Create: `e:/sourcecode/DynoVirtualAudioMixer/src/components/nodes/VuMeter.tsx`
- Create: `e:/sourcecode/DynoVirtualAudioMixer/src/components/GraphCanvas.tsx`
- Create: `e:/sourcecode/DynoVirtualAudioMixer/src/components/TopBar.tsx`

**Interfaces:**
- Consumes: `@xyflow/react`, `lucide-react`, Tauri `@tauri-apps/api/core`
- Produces: Sleek Dark theme node patchbay editor with zoom/pan and custom styled nodes

- [ ] **Step 1: Define TypeScript audio schema and state types**
Define `AudioNodeData`, `AudioPin`, `AudioConnection`, `MeterLevels` in `src/types/audio.ts`.

- [ ] **Step 2: Implement responsive VU Meter component**
Build `VuMeter.tsx` with smooth decay, peak hold indicator, and color gradient (green `-60dB` to `-12dB`, yellow `-12dB` to `-3dB`, red `0dB`).

- [ ] **Step 3: Implement custom Node components**
Build `SourceNode.tsx`, `OutputNode.tsx`, and `VirtualDeviceNode.tsx` featuring:
  - Card header with app/device icon, friendly name, and badge
  - Volume slider (0% to 150%) and Mute toggle button
  - Left/Right input and output handles (pins) with animated VU meters

- [ ] **Step 4: Implement GraphCanvas with custom Dark Theme styling**
Assemble `GraphCanvas.tsx` using `ReactFlow`, subtle dot grid background, custom node registry, and mini-map.

- [ ] **Step 5: Verify build**
Run `bun run build` and ensure zero TypeScript or bundling errors.

- [ ] **Step 6: Commit**
`git add src/ && git commit -m "feat(ui): implement custom react flow nodes and vu-meter components"`

---

### Task 8: Interactive Patchbay Wiring, Smart Auto-Pairing & Glowing Cables

**Files:**
- Create: `e:/sourcecode/DynoVirtualAudioMixer/src/components/edges/GlowingEdge.tsx`
- Create: `e:/sourcecode/DynoVirtualAudioMixer/src/hooks/useAudioGraph.ts`
- Modify: `e:/sourcecode/DynoVirtualAudioMixer/src/components/GraphCanvas.tsx`

**Interfaces:**
- Consumes: React Flow edge events, meter state
- Produces: Smart L/R auto-pairing, Bezier curve rendering with live flowing pulse animation when audio is passing

- [ ] **Step 1: Implement custom GlowingEdge component**
Create `GlowingEdge.tsx` rendering SVG Bezier curve with drop-shadow neon glow and dynamic stroke opacity driven by real-time audio volume levels.

- [ ] **Step 2: Implement Smart Stereo Auto-Pairing logic**
When user connects `pin-source-L` to `pin-dest-L`, automatically check if `pin-source-R` and `pin-dest-R` exist on both nodes and connect them automatically with a twin edge.

- [ ] **Step 3: Connect graph updates to Tauri IPC backend**
Wire edge creation, deletion, and volume changes to invoke `update_routing_graph` and `set_node_volume` in real-time.

- [ ] **Step 4: Verify in browser / build**
Run `bun run build` and verify clean build.

- [ ] **Step 5: Commit**
`git add src/ && git commit -m "feat(ui): add smart stereo auto-pairing and glowing bezier edges"`

---

### Task 9: Preset Profiles & Windows System Tray

**Files:**
- Create: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/src/tray.rs`
- Create: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/src/presets.rs`
- Modify: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/src/main.rs`

**Interfaces:**
- Consumes: Tauri Tray API, Serde JSON file I/O
- Produces: Save/Load graph presets to `%APPDATA%/DynoMixer/profiles/*.json`, minimize to tray on close

- [ ] **Step 1: Implement Profile serialization manager**
Implement JSON save and load of node positions, volumes, and connection wires.

- [ ] **Step 2: Implement System Tray menu and background minimize**
Create tray icon with menu (`Show Dyno Mixer`, `Mute All`, `Exit`). Configure window close button (`on_window_event`) to hide the window instead of killing the audio process.

- [ ] **Step 3: Verify build**
Run `cargo check --manifest-path src-tauri/Cargo.toml` and ensure compilation success.

- [ ] **Step 4: Commit**
`git add src-tauri/ && git commit -m "feat(app): add profile persistence and system tray minimization"`

---

### Task 10: End-to-End Build Pass Gate, Documentation & Verification

**Files:**
- Create: `e:/sourcecode/DynoVirtualAudioMixer/README.md`
- Create: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/tests/e2e_audio_smoke_test.rs`
- Run: Static code review & regression test suite

**Interfaces:**
- Consumes: Full workspace
- Produces: Complete verified build with zero errors, comprehensive documentation

- [ ] **Step 1: Write comprehensive README.md**
Document overview, architecture, prerequisites, usage, and keyboard shortcuts.

- [ ] **Step 2: Run all test suites**
Execute `cargo test` and `bun test` ensuring 100% pass rate with exit code 0.

- [ ] **Step 3: Execute release build verification**
Run `bun run build` and `cargo check --release` to verify compilation.

- [ ] **Step 4: Final commit**
`git add . && git commit -m "docs: complete comprehensive readme and verify full build pass gate"`
