# Frontend Node Graph Canvas (React Flow + WASAPI Patchbay)

## 1. Overview & Purpose
The Dyno Virtual Audio Mixer frontend provides a high-performance, dark-themed interactive node patchbay inspired by Rogue Amoeba's Loopback and hardware studio mixing consoles. Built on React 19, `@xyflow/react` (React Flow v12), TypeScript, Tailwind CSS, and Lucide icons, it allows users to visually route stereo and multi-channel audio between running applications, physical capture devices (microphones), virtual audio cables, and hardware output endpoints.

### Key Capabilities:
- **Interactive Patchbay Canvas**: Zoom, pan, drag-and-drop node organization, mini-map preview, and dark dot grid canvas.
- **Glassmorphic Hardware Cards**: Loopback-inspired dark aesthetic (`bg-slate-900/95 border-slate-700/60 rounded-xl shadow-xl backdrop-blur-md`).
- **Real-Time LED VU Meters**: Dual stereo channel meters with 60 FPS peak decibel monitoring (-60 dBFS to 0 dBFS) with green/yellow/red gradient and clip indicators.
- **Glowing Patch Cords**: Animated, channel-color-coded SVG wires connecting Left (sky blue) and Right (indigo) channel pins.
- **Live Gain & Mute Controls**: Non-destructive volume faders (0% to 150%) and instant mute toggle switches communicating to the WASAPI mixer engine.
- **Dynamic Node Management**: Modals to scan and inject applications (`get_running_apps`), hardware devices (`get_devices`), and provision new virtual cables (`create_virtual_device`).

---

## 2. Prerequisites & Dependencies
- **Runtime**: Bun `>= 1.1.0` or Node.js `>= 20.0`
- **Core Dependencies**:
  - `@xyflow/react`: `^12.4.2`
  - `@tauri-apps/api`: `^2.2.0`
  - `lucide-react`: `^0.477.0`
  - `react`: `^19.0.0`
  - `react-dom`: `^19.0.0`
  - `tailwindcss`: `^3.4.17`

---

## 3. Component & API Reference

### 3.1 Custom Node Components (`src/components/nodes/`)
| Component | Classification | Input Pins (Left) | Output Pins (Right) | Features |
| :--- | :--- | :--- | :--- | :--- |
| `SourceNode` | Application / Mic Input | None | `out-0` (L), `out-1` (R) | App/Mic icon, live VU meter, volume slider, mute toggle, delete button |
| `OutputNode` | Speaker / Headphone Destination | `in-0` (L), `in-1` (R) | None | Headphones/Speaker icon, live VU meter, volume slider, mute toggle, delete button |
| `VirtualDeviceNode` | Pass-through Virtual Cable | `in-0` (L), `in-1` (R) | `out-0` (L), `out-1` (R) | Driver badge, pass-through VU meter, volume slider, mute toggle, delete button |

### 3.2 VU Meter Component (`src/components/nodes/VuMeter.tsx`)
Converts decibels (-60 dBFS to 0 dBFS) to a normalized percentage:
$$\text{percent} = \text{clamp}\left(\frac{\text{dB} + 60}{60} \times 100, 0, 100\right)$$

- **Green Zone**: Below `-12 dBFS` (0% to 80% width)
- **Yellow Zone**: `-12 dBFS` to `-3 dBFS` (80% to 95% width)
- **Red Zone**: Above `-3 dBFS` (95% to 100% width)
- **Clip Indicator**: Latches red LED indicator when peak amplitude reaches $\ge -0.5\text{ dBFS}$.

### 3.3 TopBar Header & Modals (`src/components/TopBar.tsx`, `src/components/modals/`)
- `AddSourceModal`: Scans running processes via `get_running_apps` or hardware inputs via `get_devices`.
- `CreateVirtualModal`: Configures device name and driver type (Dyno Software Loopback, VB-Cable, VAC) via `create_virtual_device`.
- `AddOutputModal`: Enumerates hardware playback endpoints via `get_devices`.

---

## 4. Usage Examples

### 4.1 Embedding GraphCanvas in Application
```tsx
import React, { useRef } from "react";
import { TopBar } from "./components/TopBar";
import { GraphCanvas, type GraphCanvasHandle } from "./components/GraphCanvas";

export default function MixerApp() {
  const canvasRef = useRef<GraphCanvasHandle>(null);

  return (
    <div className="flex flex-col h-screen w-screen bg-[#0a0f1d]">
      <TopBar
        onAddSource={(p) => canvasRef.current?.addSourceNode(p)}
        onAddVirtualDevice={(p) => canvasRef.current?.addVirtualNode(p)}
        onAddOutput={(p) => canvasRef.current?.addOutputNode(p)}
      />
      <main className="flex-1 relative w-full h-full">
        <GraphCanvas ref={canvasRef} />
      </main>
    </div>
  );
}
```

### 4.2 Building & Running Tests
```bash
# Run unit tests across all components
bun test

# Build production bundle with TypeScript verification
bun run build
```

---

## 5. Output Structure & Schema

```
src/
├── types/
│   ├── audio.ts               # Domain types, DTO contracts, and React Flow payload interfaces
│   └── audio.test.ts          # Type contract verification tests
├── components/
│   ├── TopBar.tsx             # Application header with real-time telemetry and modal triggers
│   ├── GraphCanvas.tsx        # React Flow canvas container with mini-map and controls
│   ├── canvas/
│   │   ├── initialGraph.ts    # Default sample patchbay topology
│   │   ├── initialGraph.test.ts
│   │   ├── nodeFactory.ts     # Factory functions for node generation
│   │   ├── nodeFactory.test.ts
│   │   ├── useMeterStream.ts  # 60 FPS peak decibel telemetry hook
│   │   └── useMeterStream.test.ts
│   ├── modals/
│   │   ├── AddSourceModal.tsx
│   │   ├── AddSourceModal.test.ts
│   │   ├── CreateVirtualModal.tsx
│   │   └── AddOutputModal.tsx
│   └── nodes/
│       ├── VuMeter.tsx        # Stereo LED peak meter component
│       ├── VuMeter.test.ts
│       ├── SourceNode.tsx     # Custom audio input node
│       ├── OutputNode.tsx     # Custom audio playback endpoint node
│       └── VirtualDeviceNode.tsx # Custom virtual bus pass-through node
├── App.tsx                    # Root layout integrating TopBar, Canvas, and Footer
├── index.css                  # Global Tailwind CSS and typography rules
└── main.tsx                   # React root entry point
```

---

## 6. Troubleshooting & Edge Cases

| Issue | Cause | Solution |
| :--- | :--- | :--- |
| **Canvas appears blank / black** | Parent container does not have an explicit height and width. | Ensure the canvas container has `relative w-full h-full overflow-hidden`. |
| **No VU meter movement in browser** | Running in standalone Vite preview outside the Tauri window. | The built-in fallback oscillator automatically activates and simulates organic audio bounce. |
| **Wires not snapping to handles** | React Flow handle IDs mismatched. | Source handles are keyed `out-0` (Left) and `out-1` (Right); destination handles are `in-0` and `in-1`. |
| **TypeScript error TS2307 `bun:test`** | Missing `bun-types` in `tsconfig.json`. | Ensure `"types": ["bun-types"]` is configured in `tsconfig.json`. |
