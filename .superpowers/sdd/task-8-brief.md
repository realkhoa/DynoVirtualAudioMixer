### Task 8: Interactive Patchbay Wiring, Smart Auto-Pairing & Glowing Cables

**Files:**
- Create: `e:/sourcecode/DynoVirtualAudioMixer/src/components/edges/GlowingEdge.tsx`
- Create: `e:/sourcecode/DynoVirtualAudioMixer/src/components/edges/GlowingEdge.test.ts`
- Create: `e:/sourcecode/DynoVirtualAudioMixer/src/hooks/useAudioGraph.ts`
- Create: `e:/sourcecode/DynoVirtualAudioMixer/src/hooks/useAudioGraph.test.ts`
- Modify: `e:/sourcecode/DynoVirtualAudioMixer/src/components/GraphCanvas.tsx`

**Interfaces:**
- Consumes: `@xyflow/react`, `useMeterStream`, `src/types/audio.ts`
- Produces:
  - `GlowingEdge`: Custom edge component rendering smooth Bezier curves with neon glow filter, animated SVG dash strokes for active audio signal, and interactive delete button / click-to-disconnect.
  - `useAudioGraph`: Hook encapsulating:
    - `onConnect`: Detects if source pin is Left channel (`out-0`) and target pin is Left channel (`in-0`). If so, automatically adds a twin edge connecting Right channel (`out-1` -> `in-1`) if both pins exist on the source and destination nodes (Smart Stereo Auto-Pairing!).
    - `onEdgesChange`, `onNodesChange`
    - Volume and Mute debounced sync to Tauri IPC backend (`set_node_volume`)
    - Graph topology sync to Tauri IPC backend (`update_routing_graph`)
  - Integration into `GraphCanvas.tsx` with registered `edgeTypes = { glowing: GlowingEdge }`.

- [ ] **Step 1: Write unit tests for Smart Stereo Auto-Pairing and Graph Hook**
In `src/hooks/useAudioGraph.test.ts` and `src/components/edges/GlowingEdge.test.ts`:
  - Test smart auto-pairing function: connecting `out-0` to `in-0` produces 2 edges (`out-0` -> `in-0` and `out-1` -> `in-1`).
  - Test single channel connection (e.g. `out-0` to `in-1` or mono target) only produces 1 edge without false twins.
  - Test edge deletion removes edge and updates graph matrix.

- [ ] **Step 2: Run tests to verify they fail**
Run `bun test src/hooks/useAudioGraph.test.ts src/components/edges/GlowingEdge.test.ts` and observe failure.

- [ ] **Step 3: Implement GlowingEdge and useAudioGraph**
1. In `src/components/edges/GlowingEdge.tsx`:
   - Use `getBezierPath` from `@xyflow/react`.
   - Render background glow path (`stroke-cyan-500/30 filter blur(3px)` or channel-specific color).
   - Render main path with animated dash stroke when active signal > -50 dB.
   - Render mid-point interactive delete button (`x`) on hover.
2. In `src/hooks/useAudioGraph.ts`:
   - Implement `useAudioGraph` hook with smart auto-pairing, node/edge state, volume fader callbacks, and IPC synchronization.
3. In `src/components/GraphCanvas.tsx`:
   - Register custom edge type `glowing: GlowingEdge`.
   - Use `useAudioGraph` hook for all graph manipulations.
4. Ensure 100% TSDoc / Doxygen tags (`/// @brief`, `/// @param`, `/// @return`).

- [ ] **Step 4: Run tests to verify they pass**
Run `bun test` and verify all tests pass.
Run `bun run build` and verify clean build with exit code 0.

- [ ] **Step 5: Commit**
`git add src/ && git commit -m "feat(ui): add smart stereo auto-pairing and glowing bezier edges"`
