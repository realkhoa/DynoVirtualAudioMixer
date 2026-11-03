/**
 * @file EdgeInspectorModal.test.ts
 * @brief Unit tests for EdgeInspectorModal decibel conversion and gain mapping helpers.
 */

import { describe, expect, test } from "bun:test";
import { linearToDb, dbToLinear, EdgeInspectorModal } from "./EdgeInspectorModal";

describe("EdgeInspectorModal Helpers", () => {
  test("defines EdgeInspectorModal component function", () => {
    expect(typeof EdgeInspectorModal).toBe("function");
  });

  test("linearToDb converts unity gain 1.0 to 0.0 dB", () => {
    expect(linearToDb(1.0)).toBeCloseTo(0.0, 3);
  });

  test("linearToDb converts 2.0x gain to approx +6.02 dB clamped to +6.0 dB", () => {
    expect(linearToDb(2.0)).toBe(6.0);
  });

  test("linearToDb clamps silence and tiny gains <= 0.001 to -60.0 dB", () => {
    expect(linearToDb(0.0)).toBe(-60.0);
    expect(linearToDb(0.0005)).toBe(-60.0);
  });

  test("dbToLinear converts 0 dB back to 1.0", () => {
    expect(dbToLinear(0)).toBeCloseTo(1.0, 3);
  });

  test("dbToLinear converts -60 dB to 0.0 linear multiplier", () => {
    expect(dbToLinear(-60)).toBe(0.0);
    expect(dbToLinear(-70)).toBe(0.0);
  });

  test("dbToLinear converts +6 dB to ~1.995x linear multiplier", () => {
    expect(dbToLinear(6)).toBeCloseTo(1.995, 2);
  });
});
