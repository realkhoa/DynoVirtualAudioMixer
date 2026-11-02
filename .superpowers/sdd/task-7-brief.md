### Task 7: Frontend Node Graph Canvas (React Flow + Custom Nodes)

**Files:**
- Create: `e:/sourcecode/DynoVirtualAudioMixer/src/types/audio.ts`
- Create: `e:/sourcecode/DynoVirtualAudioMixer/src/components/nodes/SourceNode.tsx`
- Create: `e:/sourcecode/DynoVirtualAudioMixer/src/components/nodes/OutputNode.tsx`
- Create: `e:/sourcecode/DynoVirtualAudioMixer/src/components/nodes/VirtualDeviceNode.tsx`
- Create: `e:/sourcecode/DynoVirtualAudioMixer/src/components/nodes/VuMeter.tsx`
- Create: `e:/sourcecode/DynoVirtualAudioMixer/src/components/GraphCanvas.tsx`
- Create: `e:/sourcecode/DynoVirtualAudioMixer/src/components/TopBar.tsx`
- Modify: `e:/sourcecode/DynoVirtualAudioMixer/src/App.tsx`

**Interfaces:**
- Consumes: `@xyflow/react`, `lucide-react`, Tauri `@tauri-apps/api/core`
- Produces: Sleek Dark theme node patchbay editor with zoom/pan and custom styled nodes:
  - `SourceNode`: Card for app/mic inputs with L/R output pins, icon, volume slider, mute toggle, and Peak VU meters.
  - `OutputNode`: Card for speaker/headphone outputs with L/R input pins, volume slider, mute toggle, and Peak VU meters.
  - `VirtualDeviceNode`: Card for virtual endpoints (e.g. "Stream Mix", "Podcast") with L/R input and output pass-through pins.
  - `VuMeter`: Vertical or horizontal LED-style level meter with smooth decay and green/yellow/red gradient.
  - `TopBar`: Header with `+ Add Source` (modal listing apps/microphones), `+ New Virtual Device` (modal with name input), and `+ Add Output` (modal listing speakers/headphones).
  - `GraphCanvas`: React Flow container hosting the node graph with dark dot grid, controls, and mini-map.

- [ ] **Step 1: Define TypeScript audio schema and types**
In `src/types/audio.ts`:
  - Define `AudioDeviceDescriptor`, `AudioProcessDescriptor`, `VirtualDeviceDescriptor`.
  - Define `AudioNodeData` for React Flow node payloads (id, name, type, icon, volume, muted, channels, leftLevelDb, rightLevelDb).
  - Define `MeterLevels` map (`nodeId -> { left: number, right: number }`).

- [ ] **Step 2: Implement responsive VU Meter component**
In `src/components/nodes/VuMeter.tsx`:
  - Visual range `-60dB` to `0dB` mapped to `0%` to `100%`.
  - Green gradient below `-12dB`, yellow `-12dB` to `-3dB`, red above `-3dB`.
  - Smooth animation, peak indicator.

- [ ] **Step 3: Implement custom Node components**
In `src/components/nodes/SourceNode.tsx`, `OutputNode.tsx`, `VirtualDeviceNode.tsx`:
  - Sleek dark glassmorphism card styling (`bg-zinc-900/90 border border-zinc-700/60 rounded-xl shadow-xl`).
  - L/R connection handles (`Handle` from `@xyflow/react`) with pin labels and active VU meters.
  - Custom volume fader slider and mute button with tooltip.

- [ ] **Step 4: Implement TopBar with Action Modals**
In `src/components/TopBar.tsx`:
  - Branded logo "Dyno Audio Mixer", buttons: `+ Add Source`, `+ New Virtual Device`, `+ Add Output`.
  - Popovers / dropdowns querying `get_devices`, `get_running_apps`, `get_virtual_devices` from Tauri backend and adding nodes to graph on selection.

- [ ] **Step 5: Assemble GraphCanvas and update App.tsx**
In `src/components/GraphCanvas.tsx` and `src/App.tsx`:
  - Register `nodeTypes = { source: SourceNode, output: OutputNode, virtual: VirtualDeviceNode }`.
  - Set default sample nodes on first load (e.g., Spotify Source, Shure Mic Source, Realtek Headphones Output, Virtual Cable Output).
  - Integrate with `@xyflow/react` Background, Controls, MiniMap with dark palette.

- [ ] **Step 6: Verify build**
Run `bun run build` and ensure exit code 0 with zero TypeScript errors.

- [ ] **Step 7: Commit**
`git add src/ && git commit -m "feat(ui): implement custom react flow nodes, vu-meter, and dark canvas"`
