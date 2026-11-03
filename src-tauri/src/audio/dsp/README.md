# Dyno Audio DSP Module

Real-time low-latency Digital Signal Processing (DSP) filters for the Dyno Virtual Audio Mixer engine, featuring an exponential envelope Noise Gate and sidechain Auto-Ducking.

---

## 1. Overview & Purpose

The `dsp` module implements real-time audio filters applied across individual channels or between cross-channel sidechain triggers in Dyno's routing matrix. Designed for zero memory allocation on the audio hot path, it provides:
- **`NoiseGate`**: Attenuates ambient room noise, computer fans, and keyboard typing during quiet microphone intervals using an exponential envelope follower.
- **`AutoDucker`**: Prioritizes vocal audio over background music/game streams by dynamically attenuating target audio streams whenever speaking activity exceeds a configured voice threshold.

---

## 2. Prerequisites & Dependencies

- **Rust Runtime**: Rust 1.75.0 or higher.
- **Dependencies**:
  - `std::f32::consts::E` for exponential envelope time constants.
  - Zero third-party C library or heavy framework requirements.

---

## 3. Configuration & Parameter Reference

### Noise Gate Parameters (`NoiseGate`)
| Parameter | Type | Default | Valid Range | Description |
| :--- | :--- | :--- | :--- | :--- |
| `threshold_db` | `f32` | `-45.0` | `-80.0` to `0.0` | Decibel gate opening threshold in dBFS |
| `attack_ms` | `f32` | `5.0` | `0.1` to `100.0` | Time taken to open the gate when speech starts |
| `release_ms` | `f32` | `100.0` | `10.0` to `1000.0` | Time taken to close the gate when speech stops |
| `sample_rate` | `f32` | `48000.0` | `44100.0` to `192000.0` | Sampling frequency in Hertz |
| `enabled` | `bool` | `true` | `true` / `false` | Bypass toggle |

### Auto Ducker Parameters (`AutoDucker`)
| Parameter | Type | Default | Valid Range | Description |
| :--- | :--- | :--- | :--- | :--- |
| `threshold_db` | `f32` | `-35.0` | `-80.0` to `0.0` | Sidechain voice detection threshold in dBFS |
| `duck_attenuation_db` | `f32` | `-15.0` | `-40.0` to `0.0` | Attenuation depth applied to target streams |
| `attack_ms` | `f32` | `15.0` | `1.0` to `200.0` | Attack ramp time to lower target volume |
| `release_ms` | `f32` | `400.0` | `50.0` to `2000.0` | Release ramp time to restore full music volume |
| `sample_rate` | `f32` | `48000.0` | `44100.0` to `192000.0` | Audio stream sampling frequency |
| `enabled` | `bool` | `true` | `true` / `false` | Bypass toggle |

---

## 4. Usage Examples

### Noise Gate Example (Rust)
```rust
use dyno_virtual_audio_mixer_lib::audio::dsp::NoiseGate;

// Create 48 kHz gate with -42 dBFS threshold
let mut gate = NoiseGate::new(-42.0, 5.0, 100.0, 48000.0);

let mut audio_buffer = vec![0.001f32; 256];
gate.process(&mut audio_buffer);
// Buffer is smoothly attenuated below the threshold
```

### Sidechain Auto-Ducker Example (Rust)
```rust
use dyno_virtual_audio_mixer_lib::audio::dsp::AutoDucker;

let mut ducker = AutoDucker::new(-35.0, -15.0, 10.0, 300.0, 48000.0);

let mic_voice_buffer = vec![0.8f32; 256]; // Active talking
let mut spotify_buffer = vec![0.5f32; 256]; // Background music

ducker.process_sidechain(&mic_voice_buffer, &mut spotify_buffer);
// Spotify buffer is smoothly ducked down by -15 dB
```

---

## 5. Output Structure & Schema

Processed audio buffers preserve buffer length and sample rates in-place (`&mut [f32]`).
De-zippering interpolation is executed per-sample across the slice using linear interpolation:
$$\text{gain}(i) = \text{start\_gain} + \left(\frac{i}{N}\right) \times (\text{target\_gain} - \text{start\_gain})$$
This guarantees zero clicks, pops, or zipper noise artifacts during transitions.

---

## 6. Troubleshooting & Edge Cases

1. **Gate Chattering / Fluttering:**
   - *Symptom:* Rapid opening and closing on background breaths.
   - *Fix:* Increase `release_ms` (e.g. 150ms-250ms) to allow smoother natural speech pauses.
2. **Abrupt Music Jumps:**
   - *Symptom:* Music volume springs back too fast between spoken sentences.
   - *Fix:* Increase `release_ms` on the Auto-Ducker to `500ms`-`800ms`.
