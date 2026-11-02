### Task 2: Core Audio Engine - Lock-Free Ring Buffer & Buffer Types

**Files:**
- Modify: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/Cargo.toml` (add `rtrb = "0.3"`)
- Create: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/src/audio/mod.rs`
- Create: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/src/audio/types.rs`
- Create: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/src/audio/ring_buffer.rs`
- Test: `e:/sourcecode/DynoVirtualAudioMixer/src-tauri/tests/test_ring_buffer.rs`

**Interfaces:**
- Consumes: `rtrb` crate, standard library atomics
- Produces: `AudioChunk`, `AudioRingBuffer`, `SampleRate`, `ChannelCount`, `calculate_peak_db`

- [ ] **Step 1: Write failing unit test for ring buffer queue**
Write test `tests/test_ring_buffer.rs` verifying:
  1. Creation of `AudioRingBuffer` with capacity
  2. Single-producer single-consumer pushing and popping of `AudioChunk`
  3. Peak dB calculation (`calculate_peak_db`) where silence gives -60.0 dB (or -inf) and full scale 1.0 gives 0.0 dB
  4. Multi-threaded push/pop across producer and consumer threads without dropped frames or panics

- [ ] **Step 2: Run test to verify it fails**
Run `cargo test --manifest-path src-tauri/Cargo.toml --test test_ring_buffer` and observe failure.

- [ ] **Step 3: Implement lock-free audio buffer types**
1. In `src-tauri/Cargo.toml`, add `rtrb = "0.3"`.
2. In `src-tauri/src/audio/types.rs`:
   - Define `ChannelCount` (e.g. Stereo = 2, Mono = 1, etc.)
   - Define `SampleRate` (e.g. 44100, 48000, 96000)
   - Define `AudioChunk`: struct holding `samples: Vec<f32>`, `channels: u16`, `sample_rate: u32`.
   - Implement `calculate_peak_db(&self) -> (f32, f32)` (L/R peak in decibels).
   - Implement soft-clipping/limiter method on `AudioChunk`.
3. In `src-tauri/src/audio/ring_buffer.rs`:
   - Implement `AudioRingBuffer` wrapping `rtrb::Producer<f32>` and `rtrb::Consumer<f32>` (or `AudioChunk`).
   - Implement non-blocking write (`push_samples(&mut self, samples: &[f32]) -> usize`) and non-blocking read (`pop_samples(&mut self, dest: &mut [f32]) -> usize`).
4. In `src-tauri/src/audio/mod.rs`:
   - Export `pub mod types;` and `pub mod ring_buffer;`.
5. In `src-tauri/src/lib.rs`:
   - Add `pub mod audio;`.
6. Ensure 100% Rustdoc coverage with Doxygen tags (`/// @brief`, `/// @param`, `/// @return`).

- [ ] **Step 4: Run test to verify it passes**
Run `cargo test --manifest-path src-tauri/Cargo.toml --test test_ring_buffer` and verify exit code 0.

- [ ] **Step 5: Commit**
`git add src-tauri/ && git commit -m "feat(audio): implement lock-free ring buffer and audio chunk primitives"`
