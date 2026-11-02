/**
 * @file AddSourceModal.test.ts
 * @brief Unit tests for application icon classification and modal helpers.
 */

import { describe, expect, test } from "bun:test";
import { resolveAppIcon } from "./AddSourceModal";

describe("resolveAppIcon", () => {
  test("identifies music and streaming applications", () => {
    expect(resolveAppIcon("spotify.exe")).toBe("music");
    expect(resolveAppIcon("Tidal.exe")).toBe("music");
    expect(resolveAppIcon("applemusic.exe")).toBe("music");
  });

  test("identifies web browsers", () => {
    expect(resolveAppIcon("chrome.exe")).toBe("browser");
    expect(resolveAppIcon("msedge.exe")).toBe("browser");
    expect(resolveAppIcon("firefox.exe")).toBe("browser");
  });

  test("identifies gaming processes", () => {
    expect(resolveAppIcon("steam.exe")).toBe("game");
    expect(resolveAppIcon("cyberpunk2077.exe")).toBe("game");
    expect(resolveAppIcon("game.exe")).toBe("game");
  });

  test("falls back to generic app icon for unidentified binaries", () => {
    expect(resolveAppIcon("notepad.exe")).toBe("app");
    expect(resolveAppIcon("calc.exe")).toBe("app");
  });
});
