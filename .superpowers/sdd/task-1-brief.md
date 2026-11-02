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
