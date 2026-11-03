# Dyno Mini Bar Component

High-density floating audio mixer overlay for live streaming and gaming sessions.

---

## 1. Overview & Purpose

The `compact` component module provides `MiniBar`, a minimal desktop overlay widget. It surfaces active channels, linear volume faders, mute toggles, real-time mini peak VU meters, and an expand studio button to restore the full graph patchbay without covering the screen.

---

## 2. Prerequisites & Dependencies

- **Framework**: React 19, TypeScript 5.6.
- **Icons**: `lucide-react`.
- **Graph Types**: `@xyflow/react` and `src/types/audio.ts`.

---

## 3. Component Props & Options Reference

### `MiniBarProps`
| Prop | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `nodes` | `Node<AudioNodeData>[]` | Yes | Active mixer nodes in the patchbay graph |
| `onVolumeChange` | `(id: string, volume: number) => void` | Yes | Callback adjusting linear volume |
| `onMuteToggle` | `(id: string, muted: boolean) => void` | Yes | Callback toggling mute state |
| `onExpandToGraph` | `() => void` | Yes | Callback restoring full studio canvas |
| `onOpenDucking` | `() => void` | No | Callback opening sidechain ducking dialog |
| `isDuckingActive` | `boolean` | No | Whether sidechain auto-ducking is enabled |

---

## 4. Usage Examples

```tsx
import { MiniBar } from "./components/compact/MiniBar";

function MixerShell() {
  return (
    <MiniBar
      nodes={activeNodes}
      onVolumeChange={(id, vol) => handleVol(id, vol)}
      onMuteToggle={(id, muted) => handleMute(id, muted)}
      onExpandToGraph={() => setIsMini(false)}
      onOpenDucking={() => setIsDucking(true)}
      isDuckingActive={true}
    />
  );
}
```

---

## 5. Output Structure & Schema

The component renders fixed at bottom center (`fixed bottom-6 left-1/2 -translate-x-1/2 z-40`) with a responsive grid (`grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4`). It filters out offline devices automatically.

---

## 6. Troubleshooting & Edge Cases

1. **No active channels displayed**:
   - Occurs when all nodes are marked `isOffline`. An empty state message is shown.
2. **Overlay obscuring game clicks**:
   - The user can click "Expand Studio" or toggle the top bar "Mini Bar" button to hide the overlay.
