# Dyno Virtual Audio Mixer

High-performance, low-latency audio patchbay desktop application for Windows built with Tauri v2, Rust WASAPI audio engine, and React Flow.

## 1. Overview & Purpose

Dyno Virtual Audio Mixer provides a lightweight (<50MB RAM footprint), visually intuitive, and modular audio routing matrix inspired by Rogue Amoeba Loopback. It enables content creators, streamers, podcasters, and sound engineers to intercept, combine, process, and route individual Windows application audio streams and hardware inputs to virtual devices (for Discord, OBS, Zoom) or physical monitor outputs.

### Architecture Overview
- **UI & Presentation Layer:** React 19, TypeScript, Vite, Tailwind CSS, and `@xyflow/react` (React Flow) rendering customizable audio node graphs with real-time VU meter animations (60 FPS) and glowing Bezier signal cables.
- **IPC & Bridge Layer:** High-throughput Tauri v2 IPC channels (`invoke`, event streams) for sub-millisecond telemetry synchronization between Rust and WebView2.
- **Low-Latency Core Engine:** Native Rust audio backend utilizing lock-free SPSC ring buffers (`rtrb`), Windows WASAPI device capture and process-level loopback, stereo downmixing/upmixing, and dynamic gain routing.
- **Virtual Audio Device Pipeline:** Integration with Windows virtual audio cable drivers (VB-Cable / Sysvad WDM endpoints) to inject mixed audio streams into target communication apps.

```
┌────────────────────────────────────────────────────────┐
│               Frontend: React 19 + React Flow          │
│       (Interactive Nodes, VU Meters, Volume Sliders)   │
└───────────────────────────┬────────────────────────────┘
                            │ Tauri IPC (Commands / Events)
┌───────────────────────────▼────────────────────────────┐
│               Tauri v2 Desktop Shell (Rust)             │
├────────────────────────────────────────────────────────┤
│           Dyno Audio Engine (Rust Core)                │
│  - Lock-free Ring Buffers (rtrb)                       │
│  - WASAPI Hardware Capture & Render (cpal / windows-rs)│
│  - Process Audio Loopback Session Enumerator           │
│  - Dynamic Channel Matrix & Gain Processing            │
└────────────────────────────────────────────────────────┘
```

## 2. Prerequisites & Dependencies

### Runtime & Build Prerequisites
- **Operating System:** Windows 10 (Build 19041+) or Windows 11 x64.
- **Package Manager / JavaScript Runtime:** [Bun](https://bun.sh/) v1.3.0 or higher (Node.js v20+ optional fallback).
- **Rust Toolchain:** Stable Rust compiler (`rustc` & `cargo`) v1.75.0 or higher.
- **C++ Build Tools:** Microsoft Visual Studio C++ Build Tools with Windows 10/11 SDK (for Cpal and WASAPI COM bindings).
- **WebView2 Runtime:** Microsoft Edge WebView2 Evergreen Runtime (pre-installed on Windows 10/11).

### Key Dependencies
- **Frontend:**
  - `react` & `react-dom` (v19)
  - `@xyflow/react` (v12)
  - `tailwindcss` (v3)
  - `lucide-react`
  - `@tauri-apps/api` (v2)
- **Backend (Cargo):**
  - `tauri` (v2) with `tray-icon` feature
  - `serde` & `serde_json`
  - `tokio` (v1)
  - `cpal` & `windows-rs` (WASAPI integration)
  - `rtrb` (lock-free audio ring buffers)

## 3. CLI Options & Arguments Reference

The repository provides standard Bun and Cargo CLI commands for development, testing, and production builds:

| Command | Environment | Description |
| :--- | :--- | :--- |
| `bun run dev` | Development | Starts the local Vite development server at `http://localhost:1420`. |
| `bun run build` | Production | Executes TypeScript type checking (`tsc`) and bundles frontend assets to `dist/`. |
| `bun run preview` | Testing | Previews the production frontend bundle locally. |
| `bun run tauri dev` | Development | Boots the full desktop app with hot-reloading frontend and Rust backend. |
| `bun run tauri build` | Production | Builds the optimized release executable and MSI/NSIS installer. |
| `cargo check --manifest-path src-tauri/Cargo.toml` | CI / Lint | Verifies Rust compilation and type checking without producing binaries. |
| `cargo test --manifest-path src-tauri/Cargo.toml` | CI / Testing | Executes all Rust core unit and integration test suites. |

## 4. Usage Examples

### Installing Dependencies
```bash
# Install frontend npm dependencies via Bun
bun install
```

### Running Frontend in Development Mode
```bash
bun run dev
```

### Running the Full Tauri Desktop Application
```bash
bun run tauri dev
```

### Running Rust Compilation Check & Tests
```bash
# Check Rust backend compilation
cargo check --manifest-path src-tauri/Cargo.toml

# Run backend unit tests
cargo test --manifest-path src-tauri/Cargo.toml
```

### Packaging for Release
```bash
bun run tauri build
```

## 5. Output Structure & Schema

```
DynoVirtualAudioMixer/
├── .gitignore                  # Git ignore rules for node_modules, dist, and target
├── package.json                # Bun package manifest with frontend dependencies
├── tsconfig.json               # TypeScript compiler configuration
├── vite.config.ts              # Vite configuration with Tauri HMR support
├── tailwind.config.js          # Tailwind CSS theme and dark palette settings
├── postcss.config.js           # PostCSS configuration
├── index.html                  # HTML entry point with dark theme defaults
├── src/                        # Frontend source files
│   ├── main.tsx                # React application entry point
│   ├── App.tsx                 # Main application dashboard and patchbay container
│   └── index.css               # Tailwind utility and base style declarations
├── src-tauri/                  # Tauri v2 Rust backend
│   ├── Cargo.toml              # Rust crate manifest and dependencies
│   ├── tauri.conf.json         # Tauri v2 window and application configuration
│   ├── build.rs                # Tauri build hook for asset and IPC generation
│   ├── capabilities/           # Security capability definitions
│   │   └── default.json        # Main window permission sets
│   └── src/
│       ├── main.rs             # Desktop executable binary entry point
│       └── lib.rs              # Core library and Tauri builder initialization
└── docs/                       # Project specifications and architecture plans
    └── superpowers/plans/      # Detailed phase-by-phase implementation plans
```

## 6. Troubleshooting & Edge Cases

### Common Issues & Resolutions

1. **WebView2 Initialization Failure on Windows:**
   - *Symptom:* Application window opens blank or crashes on launch.
   - *Fix:* Ensure Microsoft Edge WebView2 runtime is installed. Download from Microsoft's official WebView2 Evergreen Bootstrapper.

2. **Port 1420 Collision:**
   - *Symptom:* `bun run dev` fails with `Port 1420 is already in use`.
   - *Fix:* Close lingering Vite instances via Task Manager or run `Get-Process node, bun | Stop-Process`.

3. **C++ Build Tools / SDK Not Found during Cargo Build:**
   - *Symptom:* `cargo check` fails with missing `link.exe` or Windows SDK headers.
   - *Fix:* Launch Visual Studio Installer, select **Desktop development with C++**, and verify the **Windows 10/11 SDK** component is installed.

4. **Audio Glitches or Buffer Underruns (XRuns):**
   - *Symptom:* Popping or clicking sounds during heavy CPU load.
   - *Fix:* Ensure thread priority for WASAPI render loops is elevated to `ProAudio` / `AVRT` (Multimedia Class Scheduler Service - MMCSS).
