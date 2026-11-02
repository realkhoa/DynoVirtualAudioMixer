/**
 * @file GlowingEdge.test.ts
 * @brief Unit tests for GlowingEdge component and Bezier glow rendering helpers.
 */

import { describe, expect, test } from "bun:test";
import {
  GlowingEdge,
  determineEdgeGlowColor,
  isAudioActive,
  calculateGlowStrokeWidth,
  getChannelIndexFromHandle,
} from "./GlowingEdge";

describe("GlowingEdge Helpers", () => {
  test("defines GlowingEdge component function", () => {
    expect(typeof GlowingEdge).toBe("function");
  });

  test("determines Left channel neon cyan glow color for handle index 0", () => {
    expect(determineEdgeGlowColor("out-0", "in-0")).toBe("#38bdf8");
    expect(determineEdgeGlowColor("out-0", "in-1")).toBe("#38bdf8");
    expect(determineEdgeGlowColor(null, "in-0")).toBe("#38bdf8");
  });

  test("determines Right channel indigo glow color for handle index 1", () => {
    expect(determineEdgeGlowColor("out-1", "in-1")).toBe("#818cf8");
    expect(determineEdgeGlowColor(null, "in-1")).toBe("#818cf8");
  });

  test("honors custom color override when supplied", () => {
    expect(determineEdgeGlowColor("out-0", "in-0", "#10b981")).toBe("#10b981");
  });

  test("identifies active audio signal exceeding threshold for animated pulse", () => {
    expect(isAudioActive(-12.5, -50.0)).toBe(true);
    expect(isAudioActive(-49.9, -50.0)).toBe(true);
    expect(isAudioActive(-50.0, -50.0)).toBe(true);
    expect(isAudioActive(-50.1, -50.0)).toBe(false);
    expect(isAudioActive(-60.0, -50.0)).toBe(false);
    expect(isAudioActive(Number.NEGATIVE_INFINITY, -50.0)).toBe(false);
    expect(isAudioActive(NaN, -50.0)).toBe(false);
  });

  test("calculates dynamic glow stroke width based on signal activity", () => {
    const idleWidth = calculateGlowStrokeWidth(false, 2);
    const activeWidth = calculateGlowStrokeWidth(true, 2);
    expect(activeWidth).toBeGreaterThan(idleWidth);
    expect(idleWidth).toBe(2);
    expect(activeWidth).toBe(3);
  });

  test("extracts channel index accurately from handle identifiers", () => {
    expect(getChannelIndexFromHandle("out-0")).toBe(0);
    expect(getChannelIndexFromHandle("in-0")).toBe(0);
    expect(getChannelIndexFromHandle("out-1")).toBe(1);
    expect(getChannelIndexFromHandle("in-1")).toBe(1);
    expect(getChannelIndexFromHandle(null)).toBe(0);
    expect(getChannelIndexFromHandle(undefined)).toBe(0);
  });
});
