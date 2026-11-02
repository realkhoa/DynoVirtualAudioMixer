/**
 * @file VuMeter.test.ts
 * @brief Unit tests for VU Meter decibel-to-percentage conversion and range calculations.
 */

import { describe, expect, test } from "bun:test";
import { dbToPercent } from "./VuMeter";

describe("VuMeter dbToPercent", () => {
  test("clamps values at and below -60 dBFS to 0%", () => {
    expect(dbToPercent(-60)).toBe(0);
    expect(dbToPercent(-70)).toBe(0);
    expect(dbToPercent(-120)).toBe(0);
  });

  test("handles non-finite decibel values gracefully", () => {
    expect(dbToPercent(-Infinity)).toBe(0);
    expect(dbToPercent(NaN)).toBe(0);
  });

  test("clamps values at and above 0 dBFS to 100%", () => {
    expect(dbToPercent(0)).toBe(100);
    expect(dbToPercent(3)).toBe(100);
    expect(dbToPercent(12)).toBe(100);
  });

  test("accurately calculates standard midpoint and threshold values", () => {
    // -30 dBFS is halfway between -60 and 0 -> 50%
    expect(dbToPercent(-30)).toBeCloseTo(50, 4);

    // -12 dBFS is 48/60 -> 80% (green to yellow threshold)
    expect(dbToPercent(-12)).toBeCloseTo(80, 4);

    // -3 dBFS is 57/60 -> 95% (yellow to red threshold)
    expect(dbToPercent(-3)).toBeCloseTo(95, 4);

    // -6 dBFS is 54/60 -> 90%
    expect(dbToPercent(-6)).toBeCloseTo(90, 4);
  });
});
